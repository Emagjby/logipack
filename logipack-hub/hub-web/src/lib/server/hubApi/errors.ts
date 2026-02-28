import type { HttpMethod } from "./types";

export type HubApiUpstream = {
	method?: HttpMethod;
	path?: string;
};

export type HubApiErrorInit = {
	status: number;
	code: string;
	message: string;
	retryable: boolean;
	upstream?: HubApiUpstream;
	cause?: unknown;
};

export class HubApiError extends Error {
	status: number;
	code: string;
	retryable: boolean;
	upstream?: HubApiUpstream;
	cause?: unknown;

	constructor(init: HubApiErrorInit) {
		super(init.message);
		this.name = "HubApiError";
		Object.setPrototypeOf(this, HubApiError.prototype);
		this.status = init.status;
		this.code = init.code;
		this.retryable = init.retryable;
		this.upstream = init.upstream;
		this.cause = init.cause;
	}
}

// Helpers:

function isRecord(v: unknown): v is Record<string, unknown> {
	return !!v && typeof v === "object" && !Array.isArray(v);
}

function retryableByStatus(status: number): boolean {
	if (status >= 500) return true;
	return false;
}

/**
 * Parse backend error JSON shape `{ code: message }`.
 * Returns null if body isn't that shape.
 */
export function parseBackendErrorJson(
	body: unknown,
): { code: string; message: string } | null {
	if (!isRecord(body)) return null;
	const code = body.code;
	const message = body.message;

	if (typeof code !== "string" || !code) return null;
	if (typeof message !== "string" || !message) return null;

	return { code, message };
}

/**
 * Normalize an HTTP error response into HubApiError.
 * - Attempts JSON parse first.
 * - Supports backend `{ code, message }`.
 * - Falls back to body text / generic message.
 */
export async function hubApiErrorFromResponse(args: {
	response: Response;
	upstream?: HubApiUpstream;
}): Promise<HubApiError> {
	const { response, upstream } = args;
	const status = response.status;

	const forText = response.clone();

	let parsedJson: unknown | null = null;
	let jsonParseFailed = false;

	try {
		parsedJson = await response.json();
	} catch {
		jsonParseFailed = true;
	}

	if (parsedJson !== null && !jsonParseFailed) {
		const backend = parseBackendErrorJson(parsedJson);
		if (backend) {
			return new HubApiError({
				status,
				code: backend.code,
				message: backend.message,
				retryable: retryableByStatus(status),
				upstream,
			});
		}

		return new HubApiError({
			status,
			code: "UPSTREAM_ERROR_JSON",
			message: `upstream returned error status ${status}`,
			retryable: retryableByStatus(status),
			upstream,
			cause: parsedJson,
		});
	}

	let bodyText: string | null = null;
	try {
		bodyText = await forText.text();
	} catch {
		bodyText = null;
	}

	const hasText = typeof bodyText === "string" && bodyText.trim() !== "";

	return new HubApiError({
		status,
		code: hasText ? "UPSTREAM_ERROR_TEXT" : "UPSTREAM_ERROR",
		message: hasText
			? bodyText!.trim()
			: `upstream returned error status ${status}`,
		retryable: retryableByStatus(status),
		upstream,
		cause: jsonParseFailed
			? new SyntaxError("failed to parse error body as JSON")
			: undefined,
	});
}

/**
 * Normalize failures that happen before a Response exists:
 * - Abort / timeout
 * - Network / fetch TypeError
 * - Unknown throwables
 */
export function hubApiErrorFromThrowable(args: {
	err: unknown;
	upstream?: HubApiUpstream;
}): HubApiError {
	const { err, upstream } = args;

	if (err instanceof DOMException && err.name === "AbortError") {
		return new HubApiError({
			status: 0,
			code: "ABORTED",
			message: "request was aborted",
			retryable: true,
			upstream,
			cause: err,
		});
	}

	if (err instanceof TypeError) {
		return new HubApiError({
			status: 0,
			code: "NETWORK_ERROR",
			message: err.message || "network error",
			retryable: true,
			upstream,
			cause: err,
		});
	}

	if (err instanceof SyntaxError) {
		return new HubApiError({
			status: 0,
			code: "JSON_PARSE_FAILED",
			message: err.message || "failed to parse JSON",
			retryable: false,
			upstream,
			cause: err,
		});
	}

	return new HubApiError({
		status: 0,
		code: "UNKNOWN_ERROR",
		message: err instanceof Error ? err.message : "unknown error",
		retryable: true,
		upstream,
		cause: err,
	});
}

/**
 * Wrap JSON parsing of a successful response.
 * If parsing fails, throw a HubApiError describing JSON parse failure.
 */
export async function parseJsonOrThrowHubApiError<T>(args: {
	response: Response;
	upstream?: HubApiUpstream;
}): Promise<T> {
	const { response, upstream } = args;

	try {
		return (await response.json()) as T;
	} catch (error) {
		throw new HubApiError({
			status: response.status,
			code: "JSON_PARSE_FAILED",
			message: "failed to parse response body as JSON",
			retryable: response.status >= 500,
			upstream,
			cause: error instanceof Error ? error : undefined,
		});
	}
}
