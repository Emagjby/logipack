import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("createHubApiClient", () => {
	test("auth header attached when token exists", async () => {
		const fetchMock = makeFetchMock(async (_url, init) => {
			expect(init?.headers).toBeTruthy();
			const headers = init!.headers as Record<string, string>;
			expect(headers.Authorization).toBe("Bearer tok");
			return new Response(JSON.stringify({ ok: true }), { status: 200 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: {
				session: {
					access_token: "tok",
					refresh_token: "rt",
					expires_at: 1,
					role: "",
					name: "",
					email: "",
				},
			} as any,
			baseUrl: "https://example.com",
			timeoutMs: 50_000,
		});

		const res = await client.get<{ ok: boolean }>("/x");
		expect(res.ok).toBe(true);
		expect(res.data.ok).toBe(true);
	});

	test("auth header omitted when token missing", async () => {
		const fetchMock = makeFetchMock(async (_url, init) => {
			const headers = init!.headers as Record<string, string>;
			expect(headers.Authorization).toBeUndefined();
			return new Response(JSON.stringify({ ok: true }), { status: 200 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		const res = await client.get<{ ok: boolean }>("/x");
		expect(res.ok).toBe(true);
	});

	test("body serialization sets content-type and JSON string body", async () => {
		const fetchMock = makeFetchMock(async (_url, init) => {
			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");
			expect(init!.body).toBe(JSON.stringify({ a: 1 }));
			return new Response(JSON.stringify({ ok: true }), { status: 200 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		await client.post<{ ok: boolean }>("/x", { a: 1 }, { auth: false });
	});

	test("no cookie forwarding (explicit cookie header) -> HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			throw new Error("should not reach fetch");
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		await expect(
			client.get("/x", { headers: { Cookie: "lp_session=lol" } }),
		).rejects.toBeInstanceOf(HubApiError);
	});

	test("success JSON path", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(JSON.stringify({ value: 42 }), { status: 200 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		const res = await client.get<{ value: number }>("/x");
		expect(res.data.value).toBe(42);
	});

	test("mapped error path (400 JSON {code,message})", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(JSON.stringify({ code: "BAD", message: "nope" }), {
				status: 400,
			});
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		await expect(client.get("/x")).rejects.toBeInstanceOf(HubApiError);

		try {
			await client.get("/x");
			throw new Error("expected to throw");
		} catch (err: any) {
			expect(err.name).toBe("HubApiError");
			expect(err.message).toBe("nope");
			expect(err.status).toBe(400);
			expect(err.code).toBe("BAD");
			expect(err.retryable).toBe(false);
		}
	});

	test("raw TypeError should not leak (network error normalized)", async () => {
		const fetchMock = makeFetchMock(async () => {
			throw new TypeError("network down");
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		await expect(client.get("/x")).rejects.toBeInstanceOf(HubApiError);
	});

	test("raw SyntaxError should not leak (JSON parse failure normalized)", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response("not-json", {
				status: 200,
				headers: { "content-type": "application/json" },
			});
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: null } as any,
			baseUrl: "https://example.com",
		});

		await expect(client.get("/x")).rejects.toBeInstanceOf(HubApiError);
	});
});
