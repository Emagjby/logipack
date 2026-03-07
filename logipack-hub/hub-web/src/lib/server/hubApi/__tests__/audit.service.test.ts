import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import { listAuditEvents } from "../services/audit";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("audit service", () => {
	test("listAuditEvents sends limit and cursor query params", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/audit?limit=10&cursor=cursor-1");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					events: [],
					page: {
						limit: 10,
						next_cursor: null,
						has_next: false,
					},
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const result = await listAuditEvents(client, {
			limit: 10,
			cursor: "cursor-1",
		});
		expect(result.events).toHaveLength(0);
		expect(result.page.has_next).toBe(false);
	});

	test("listAuditEvents propagates HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(
				JSON.stringify({ code: "access_denied", message: "Access denied" }),
				{
					status: 403,
					headers: { "content-type": "application/json" },
				},
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(listAuditEvents(client)).rejects.toBeInstanceOf(HubApiError);
	});
});
