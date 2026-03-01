import type { HttpMethod, HubApiRequestOptions, HubApiSuccess } from "./types";
import { getAccessTokenFromLocals } from "./auth";
import {
	HubApiError,
	hubApiErrorFromResponse,
	hubApiErrorFromThrowable,
	parseJsonOrThrowHubApiError,
} from "./errors";

type CreateHubApiClientArgs = {
	fetch: typeof globalThis.fetch;
	locals: App.Locals;
	baseUrl: string;
	timeoutMs?: number;
};

type RequestArgs = Omit<HubApiRequestOptions, "baseUrl"> & {
	/**
	 * If provided, overrides client timeoutMs
	 */
	timeoutMs?: number;
};

function joinUrl(baseUrl: string, path: string): string {
	const base = baseUrl.replace(/\/+$/, "");
	const p = path.startsWith("/") ? path : `/${path}`;
	return `${base}${p}`;
}

function hasOwnCookieHeader(headers?: Record<string, string>): boolean {
	if (!headers) return false;
	return Object.keys(headers).some((k) => k.toLowerCase() === "cookie");
}

export function createHubApiClient(args: CreateHubApiClientArgs) {
	const { fetch, locals, baseUrl } = args;
	const defaultTimeoutMs = args.timeoutMs ?? 10_000;

	async function request<T>(req: RequestArgs): Promise<HubApiSuccess<T>> {
		const method: HttpMethod = req.method;
		const url = joinUrl(baseUrl, req.path);

		if (hasOwnCookieHeader(req.headers)) {
			throw new HubApiError({
				status: 0,
				code: "COOKIE_FORWARDING_FORBIDDEN",
				message: "cookie header forwarding to hub-api is forbidden",
				retryable: false,
				upstream: { method, path: req.path },
			});
		}

		const upstream = { method, path: req.path };

		const controller = new AbortController();
		const timeoutMs = req.timeoutMs ?? defaultTimeoutMs;
		const timeout = setTimeout(() => controller.abort(), timeoutMs);

		const headers: Record<string, string> = {
			Accept: "application/json",
			...(req.headers ?? {}),
		};

		const wantsAuth = req.auth !== false;
		if (wantsAuth) {
			const token = getAccessTokenFromLocals(locals);
			if (token) headers.Authorization = `Bearer ${token}`;
		}

		let body: string | undefined;
		if (req.body !== undefined) {
			headers["Content-Type"] = "application/json";
			try {
				body = JSON.stringify(req.body);
			} catch (err) {
				clearTimeout(timeout);
				throw new HubApiError({
					status: 0,
					code: "JSON_SERIALIZE_FAILED",
					message: "failed to serialize request body as JSON",
					retryable: false,
					upstream,
					cause: err,
				});
			}
		}

		try {
			const res = await fetch(url, {
				method,
				headers,
				body,
				signal: controller.signal,
			});

			if (!res.ok) {
				throw await hubApiErrorFromResponse({ response: res, upstream });
			}

			if (res.status === 204) {
				return {
					ok: true,
					status: res.status,
					data: undefined as unknown as T,
				};
			}

			const data = await parseJsonOrThrowHubApiError<T>({
				response: res,
				upstream,
			});

			return { ok: true, status: res.status, data };
		} catch (err) {
			if (err instanceof HubApiError) throw err;

			throw hubApiErrorFromThrowable({ err, upstream });
		} finally {
			clearTimeout(timeout);
		}
	}

	return {
		request,
		get: <T>(path: string, opts?: Omit<RequestArgs, "method" | "path">) =>
			request<T>({ method: "GET", path, ...(opts ?? {}) }),

		post: <T>(
			path: string,
			body?: unknown,
			opts?: Omit<RequestArgs, "method" | "path" | "body">,
		) => request<T>({ method: "POST", path, body, ...(opts ?? {}) }),

		put: <T>(
			path: string,
			body?: unknown,
			opts?: Omit<RequestArgs, "method" | "path" | "body">,
		) => request<T>({ method: "PUT", path, body, ...(opts ?? {}) }),

		delete: <T>(path: string, opts?: Omit<RequestArgs, "method" | "path">) =>
			request<T>({ method: "DELETE", path, ...(opts ?? {}) }),
	};
}
