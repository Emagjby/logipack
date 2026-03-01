import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import { ensureUser, getMe } from "../services/identity";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("identity service", () => {
	test("ensureUser success 204 no content", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/ensure-user");
			expect(init?.method).toBe("POST");
			expect(init?.headers).toBeTruthy();

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");
			expect(headers.Authorization).toBe("Bearer tok");

			expect(init?.body).toBe(
				JSON.stringify({ email: "a@b.com", name: "testname" }),
			);

			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await ensureUser(client, { email: "a@b.com", name: "testname" });
	});

	test("getMeRole success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/me");
			expect(init?.method).toBe("GET");

			return new Response(JSON.stringify({ role: "admin" }), { status: 200 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const role = await getMe(client);
		expect(role).toBe("admin");
	});

	test("normalized error propagation", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(JSON.stringify({ SOME_CODE: "boom" }), {
				status: 500,
			});
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		let threw = false;
		try {
			await getMe(client);
		} catch (e) {
			threw = true;
			expect(e).toBeInstanceOf(HubApiError);
			expect((e as HubApiError).status).toBe(500);
		}
		expect(threw).toBe(true);
	});
});
