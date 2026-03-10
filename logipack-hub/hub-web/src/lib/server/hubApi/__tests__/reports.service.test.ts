import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import { getReport } from "../services/reports";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("reports service", () => {
	test("getReport forwards filters and period bucket", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain(
				"/reports/shipments-by-period?from=2026-03-01&to=2026-03-10&bucket=week",
			);
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					report_name: "shipments-by-period",
					generated_at: "2026-03-10T09:00:00.000Z",
					columns: ["bucket_start", "shipment_count"],
					rows: [["2026-03-03", 4]],
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const report = await getReport(client, "shipments-by-period", {
			from: "2026-03-01",
			to: "2026-03-10",
			bucket: "week",
		});
		expect(report.rows[0]).toEqual(["2026-03-03", 4]);
	});

	test("getReport propagates HubApiError", async () => {
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

		await expect(getReport(client, "shipments-by-status")).rejects.toBeInstanceOf(
			HubApiError,
		);
	});
});
