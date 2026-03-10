import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import { getAdminOverview, getEmployeeOverview } from "../services/analytics";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("analytics service", () => {
	test("getAdminOverview forwards span query", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/analytics/admin/overview?span=90d");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					total_shipments: 12,
					shipments_vs_last_period: 3,
					shipments_timeseries: [],
					total_clients: 8,
					clients_vs_last_period: 1,
					clients_timeseries: [],
					total_offices: 4,
					offices_vs_last_period: 0,
					offices_timeseries: [],
					total_employees: 7,
					assigned_employees: 5,
					unassigned_employees: 2,
					employees_timeseries: [],
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const overview = await getAdminOverview(client, "90d");
		expect(overview.total_employees).toBe(7);
	});

	test("getEmployeeOverview uses default span", async () => {
		const fetchMock = makeFetchMock(async (url) => {
			expect(String(url)).toContain("/analytics/employee/overview?span=30d");

			return new Response(
				JSON.stringify({
					active_shipments: 6,
					active_vs_last_period: 2,
					active_timeseries: [],
					pending_shipments: 3,
					pending_vs_last_period: -1,
					pending_timeseries: [],
					deliveries_today: 2,
					deliveries_vs_last_period: 1,
					deliveries_timeseries: [],
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const overview = await getEmployeeOverview(client);
		expect(overview.active_shipments).toBe(6);
	});

	test("analytics services propagate HubApiError", async () => {
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

		await expect(getAdminOverview(client)).rejects.toBeInstanceOf(HubApiError);
	});
});
