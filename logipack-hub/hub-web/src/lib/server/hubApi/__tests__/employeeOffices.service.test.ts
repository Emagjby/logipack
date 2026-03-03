import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import {
	assignEmployeeOffice,
	listEmployeeOffices,
	removeEmployeeOffice,
} from "../services/employeeOffices";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("employeeOffices service", () => {
	test("listEmployeeOffices success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees/e1/offices");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					employee_id: "e1",
					office_ids: ["o1"],
					offices: [
						{
							id: "o1",
							name: "Sofia HQ",
							city: "Sofia",
							address: "12 Vitosha",
							updated_at: "2026-03-03T12:00:00.000Z",
						},
						{
							id: "o2",
							name: "Plovdiv DC",
							city: "Plovdiv",
							address: "Main 2",
							updated_at: "2026-03-03T12:00:00.000Z",
						},
					],
					employee: {
						id: "e1",
						user_id: "u1",
						user: { id: "u1", email: "a@b.com", name: "Alice" },
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

		const res = await listEmployeeOffices(client, "e1");
		expect(res.state).toBe("ok");
		if (res.state !== "ok") throw new Error("expected ok");

		expect(res.employee.id).toBe("e1");
		expect(res.employee.office_ids).toEqual(["o1"]);
		expect(res.currentOfficeId).toBe("o1");
		expect(res.currentOffice?.id).toBe("o1");
		expect(res.offices).toHaveLength(2);
	});

		test("assignEmployeeOffice success", async () => {
			const fetchMock = makeFetchMock(async (url, init) => {
				expect(String(url)).toContain("/admin/employees/e1/offices");
				expect(init?.method).toBe("POST");

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");
			expect(init!.body).toBe(JSON.stringify({ office_id: "o1" }));

			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await assignEmployeeOffice(client, "e1", "o1");
	});

	test("removeEmployeeOffice success 204", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees/e1/offices/o1");
			expect(init?.method).toBe("DELETE");
			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await removeEmployeeOffice(client, "e1", "o1");
	});

	test("error propagation: 400/403/404/500 -> HubApiError", async () => {
		const mkClient = (status: number, body?: unknown) => {
			const fetchMock = makeFetchMock(async () => {
				if (body !== undefined) {
					return new Response(JSON.stringify(body), {
						status,
						headers: { "content-type": "application/json" },
					});
				}
				return new Response("", { status });
			});

			return createHubApiClient({
				fetch: fetchMock,
				locals: { session: { access_token: "tok" } } as any,
				baseUrl: "https://example.com",
			});
		};

		await expect(
			listEmployeeOffices(mkClient(500), "e1"),
		).rejects.toBeInstanceOf(HubApiError);
		await expect(
			assignEmployeeOffice(mkClient(403), "e1", "o1"),
		).rejects.toBeInstanceOf(HubApiError);
		await expect(
			removeEmployeeOffice(mkClient(404), "e1", "o1"),
		).rejects.toBeInstanceOf(HubApiError);
		await expect(
			assignEmployeeOffice(
				mkClient(400, { code: "BAD", message: "nope" }),
				"e1",
				"o1",
			),
		).rejects.toBeInstanceOf(HubApiError);
	});
});
