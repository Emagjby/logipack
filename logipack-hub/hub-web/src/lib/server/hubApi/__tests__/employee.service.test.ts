import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import {
	createEmployee,
	deleteEmployee,
	getEmployee,
	listEmployees,
	updateEmployee,
} from "../services/employees";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("employees service", () => {
	test("listEmployees success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					employees: [
						{
							id: "emp-1",
							user_id: "user-1",
							full_name: "Ivan Ivanov",
							email: "ivan@logipack.dev",
							user_display_name: "Ivan",
							offices: [
								{
									id: "office-1",
									name: "Sofia HQ",
									city: "Sofia",
									address: "12 Vitosha Blvd",
									updated_at: "2026-03-01T10:00:00.000Z",
								},
							],
						},
					],
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const employees = await listEmployees(client);
		expect(employees).toHaveLength(1);
		expect(employees[0]!.id).toBe("emp-1");
	});

	test("getEmployee success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees/emp-9");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					employee: {
						id: "emp-9",
						user_id: "user-9",
						full_name: "Detail Person",
						email: "detail@logipack.dev",
						user_display_name: null,
						offices: [
							{
								id: "office-2",
								name: "Varna Port",
								city: "Varna",
								address: "8 Primorski Blvd",
								updated_at: "2026-03-01T10:00:00.000Z",
							},
						],
						created_at: "2026-03-01T09:00:00.000Z",
						updated_at: "2026-03-02T09:00:00.000Z",
						deleted_at: null,
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

		const emp = await getEmployee(client, "emp-9");
		expect(emp.id).toBe("emp-9");
		expect(emp.office_name).toBe("Varna Port");
	});

	test("createEmployee success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees");
			expect(init?.method).toBe("POST");

			return new Response(
				JSON.stringify({
					employee: {
						id: "emp-10",
						user_id: "user-10",
						full_name: "New Employee",
						email: "new@logipack.dev",
						user_display_name: null,
						created_at: "2026-03-01T09:00:00.000Z",
						updated_at: "2026-03-01T09:00:00.000Z",
						deleted_at: null,
					},
				}),
				{ status: 201 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const employee = await createEmployee(client, {
			email: "new@logipack.dev",
		});
		expect(employee.id).toBe("emp-10");
		expect(employee.email).toBe("new@logipack.dev");
	});

	test("updateEmployee success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees/emp-11");
			expect(init?.method).toBe("PUT");

			return new Response(
				JSON.stringify({
					employee: {
						id: "emp-11",
						user_id: "user-11",
						full_name: "Updated Employee",
						email: "updated@logipack.dev",
						user_display_name: null,
						created_at: "2026-03-01T09:00:00.000Z",
						updated_at: "2026-03-02T09:00:00.000Z",
						deleted_at: null,
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

		const employee = await updateEmployee(client, "emp-11", {
			email: "updated@logipack.dev",
		});
		expect(employee.id).toBe("emp-11");
		expect(employee.email).toBe("updated@logipack.dev");
	});

	test("deleteEmployee success (204)", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/employees/emp-1");
			expect(init?.method).toBe("DELETE");
			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await deleteEmployee(client, "emp-1");
	});

	test("error propagation: 404 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(
			async () => new Response("", { status: 404 }),
		);

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(getEmployee(client, "missing")).rejects.toBeInstanceOf(
			HubApiError,
		);
	});

	test("error propagation: 403 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(
			async () => new Response("", { status: 403 }),
		);

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(listEmployees(client)).rejects.toBeInstanceOf(HubApiError);
	});

	test("error propagation: 400 JSON -> HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(
				JSON.stringify({ code: "BAD_INPUT", message: "nope" }),
				{
					status: 400,
					headers: { "content-type": "application/json" },
				},
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(getEmployee(client, "bad")).rejects.toBeInstanceOf(
			HubApiError,
		);
	});

	test("error propagation: 500 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(
			async () => new Response("boom", { status: 500 }),
		);

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(listEmployees(client)).rejects.toBeInstanceOf(HubApiError);
	});
});
