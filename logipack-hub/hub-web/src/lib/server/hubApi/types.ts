export type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

export type HubApiRequestOptions = {
	method: HttpMethod;
	path: string;

	/**
	 * Absolute base URL for hub-api
	 * The client will join baseUrl + path.
	 */
	baseUrl?: string;

	/**
	 * Request timeout in ms.
	 * HTTP client will enforce via AbortController.
	 */
	timeoutMs?: number;

	/**
	 * JSON-serializable request body.
	 * Client will set Content-Type and JSON.stringify auto.
	 */
	body?: unknown;

	/**
	 * Extra headers to send with the request.
	 */
	headers?: Record<string, string>;

	/**
	 * Override whether this request should include bearer auth.
	 * Default to true, and client will include bearer token if available.
	 */
	auth?: boolean;
};

export type HubApiSuccess<T> = {
	ok: true;
	status: number;
	data: T;
};

export type HubApiFailure = {
	ok: false;
	status: number;
	code: string;
	message: string;
	retryable: boolean;
	/**
	 * Optional upstream context, used for debugging.
	 * Keep this minimal and avoid leaking secrets.
	 */
	upstream?: {
		method?: HttpMethod;
		path?: string;
	};
};
