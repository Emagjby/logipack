import { describe, expect, mock, test } from "bun:test";

const HUB_API_BASE = "https://hub-api.test";
process.env.HUB_API_BASE = HUB_API_BASE;
mock.module("$env/static/private", () => ({ HUB_API_BASE }));

type ResponseFactory = (request: Request) => Response | Promise<Response>;

function jsonResponse(body: unknown, status = 200): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { "content-type": "application/json" },
	});
}

function noContent(status = 204): Response {
	return new Response(null, { status });
}

function makeFetchMock(
	routes: Record<string, ResponseFactory>,
): typeof globalThis.fetch {
	return (async (input: RequestInfo | URL, init?: RequestInit) => {
		const request =
			input instanceof Request
				? input
				: new Request(
						input instanceof URL ? input.toString() : String(input),
						init,
					);
		const url = new URL(request.url);
		const key = `${request.method.toUpperCase()} ${url.pathname}${url.search}`;
		const handler = routes[key];

		if (!handler) {
			throw new Error(`unexpected fetch: ${key}`);
		}

		return await handler(request);
	}) as typeof globalThis.fetch;
}

function makeSession(
	role: "admin" | "employee" | "",
	extra: Record<string, unknown> = {},
) {
	return {
		access_token: "tok",
		refresh_token: "refresh",
		expires_at: 4_102_444_800,
		role,
		name: role === "admin" ? "Admin User" : "Employee User",
		email: role === "admin" ? "admin@logipack.dev" : "employee@logipack.dev",
		...extra,
	};
}

function makeLocals(
	role: "admin" | "employee" | "",
	extraSession: Record<string, unknown> = {},
): App.Locals {
	return {
		lang: "en",
		session: makeSession(role, extraSession),
	} as App.Locals;
}

function makeFormRequest(fields: Record<string, string>): Request {
	const formData = new FormData();
	for (const [key, value] of Object.entries(fields)) {
		formData.set(key, value);
	}

	return new Request("https://app.test/form", {
		method: "POST",
		body: formData,
	});
}

async function expectRedirect(
	run: () => unknown | Promise<unknown>,
	expectedLocation: string,
	expectedStatus = 303,
): Promise<void> {
	try {
		await run();
		throw new Error("expected redirect");
	} catch (error) {
		const redirect = error as { status?: number; location?: string };
		expect(redirect.status).toBe(expectedStatus);
		expect(redirect.location).toBe(expectedLocation);
	}
}

async function expectHttpError(
	run: () => unknown | Promise<unknown>,
	expectedStatus: number,
	expectedMessage: string,
): Promise<void> {
	try {
		await run();
		throw new Error("expected http error");
	} catch (error) {
		const httpError = error as { status?: number; body?: { message?: string } };
		expect(httpError.status).toBe(expectedStatus);
		expect(httpError.body?.message).toBe(expectedMessage);
	}
}

describe("app route smoke", () => {
	test("app layout keeps login and no-access redirects stable", async () => {
		const { load } = await import("../+layout.server.ts");

		await expectRedirect(
			() =>
				load({
					locals: { lang: "en", session: null } as App.Locals,
					params: { lang: "en" },
					url: new URL("https://app.test/en/app/admin"),
				} as any),
			"/en/login",
		);

		await expectRedirect(
			() =>
				load({
					locals: makeLocals(""),
					params: { lang: "en" },
					url: new URL("https://app.test/en/app/admin"),
				} as any),
			"/en/app/no-access",
		);

		const noAccess = (await load({
			locals: makeLocals(""),
			params: { lang: "en" },
			url: new URL("https://app.test/en/app/no-access"),
		} as any)) as any;

		expect(noAccess.pathname).toBe("/en/app/no-access");
	});

	test("app landing redirect preserves admin, employee, and no-access routing", async () => {
		const { load } = await import("../+page.server.ts");

		await expectRedirect(
			() =>
				load({
					locals: makeLocals("admin"),
					params: { lang: "en" },
				} as any),
			"/en/app/admin",
			302,
		);

		await expectRedirect(
			() =>
				load({
					locals: makeLocals("employee"),
					params: { lang: "en" },
				} as any),
			"/en/app/employee",
			302,
		);

		await expectRedirect(
			() =>
				load({
					locals: makeLocals(""),
					params: { lang: "en" },
				} as any),
			"/en/app/no-access",
			302,
		);
	});

	test("admin offices list load reads backend data and keeps query filtering", async () => {
		const { load } = await import("../admin/offices/+page.server.ts");
		const fetch = makeFetchMock({
			"GET /admin/offices": () =>
				jsonResponse({
					offices: [
						{
							id: "office-sofia",
							name: "Sofia HQ",
							city: "Sofia",
							address: "1 Vitosha Blvd",
							updated_at: "2026-03-07T10:00:00.000Z",
						},
						{
							id: "office-varna",
							name: "Varna Port",
							city: "Varna",
							address: "8 Primorski Blvd",
							updated_at: "2026-03-07T10:00:00.000Z",
						},
					],
				}),
		});

		const result = (await load({
			url: new URL("https://app.test/en/app/admin/offices?q=varna"),
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result.loadError).toBe(false);
		expect(result.query).toBe("varna");
		expect(result.offices).toHaveLength(1);
		expect(result.offices[0]?.id).toBe("office-varna");
	});

	test("admin offices delete action redirects to admin offices list", async () => {
		const { actions } = await import("../admin/offices/[id]/+page.server.ts");
		const fetch = makeFetchMock({
			"DELETE /admin/offices/office-sofia": () => noContent(),
		});

		await expectRedirect(
			() =>
				actions.delete({
					params: { lang: "en", id: "office-sofia" },
					fetch,
					locals: makeLocals("admin"),
				} as any),
			"/en/app/admin/offices",
		);
	});

	test("admin clients list load filters backend results", async () => {
		const { load } = await import("../admin/clients/+page.server.ts");
		const fetch = makeFetchMock({
			"GET /admin/clients": () =>
				jsonResponse({
					clients: [
						{ id: "client-acme", name: "ACME Corp", email: "ops@acme.dev" },
						{ id: "client-nova", name: "Nova Trade", email: "hello@nova.dev" },
					],
				}),
		});

		const result = (await load({
			url: new URL("https://app.test/en/app/admin/clients?q=nova"),
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result.loadError).toBe(false);
		expect(result.clients).toHaveLength(1);
		expect(result.clients[0]?.id).toBe("client-nova");
	});

	test("admin clients delete action redirects to app-scoped clients list", async () => {
		const { actions } = await import("../admin/clients/[id]/+page.server.ts");
		const fetch = makeFetchMock({
			"DELETE /admin/clients/client-acme": () => noContent(),
		});

		await expectRedirect(
			() =>
				actions.delete({
					params: { lang: "en", id: "client-acme" },
					fetch,
					locals: makeLocals("admin"),
				} as any),
			"/en/app/admin/clients",
		);
	});

	test("admin employees list load returns backend-backed rows", async () => {
		const { load } = await import("../admin/employees/+page.server.ts");
		const fetch = makeFetchMock({
			"GET /admin/employees": () =>
				jsonResponse({
					employees: [
						{
							id: "emp-1",
							user_id: "user-1",
							full_name: "Ivan Ivanov",
							email: "ivan@logipack.dev",
							user_display_name: "Ivan",
							offices: [
								{
									id: "office-sofia",
									name: "Sofia HQ",
									city: "Sofia",
									address: "1 Vitosha Blvd",
									updated_at: "2026-03-07T10:00:00.000Z",
								},
							],
						},
					],
				}),
		});

		const result = (await load({
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result.loadError).toBe(false);
		expect(result.employees).toHaveLength(1);
		expect(result.employees[0]?.office_name).toBe("Sofia HQ");
	});

	test("admin employee create action maps backend 404 to the email_not_found key", async () => {
		const { actions } = await import("../admin/employees/new/+page.server.ts");
		const fetch = makeFetchMock({
			"POST /admin/employees": () =>
				jsonResponse(
					{ code: "USER_NOT_FOUND", message: "No user found" },
					404,
				),
		});

		const result = (await actions.default({
			request: makeFormRequest({ email: "missing@logipack.dev" }),
			params: { lang: "en" },
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result?.status).toBe(404);
		expect(result?.data?.fieldErrors?.email).toBe("employee.form.email_not_found");
	});

	test("admin shipments list load reads backend shipments and offices", async () => {
		const { load } = await import("../admin/shipments/+page.server.ts");
		const fetch = makeFetchMock({
			"GET /shipments": () =>
				jsonResponse([
					{
						id: "SHP-1001",
						client_id: "client-acme",
						current_status: "NEW",
						current_office_id: "office-sofia",
						created_at: "2026-03-07T10:00:00.000Z",
						updated_at: "2026-03-07T11:00:00.000Z",
					},
				]),
			"GET /admin/offices": () =>
				jsonResponse({
					offices: [
						{
							id: "office-sofia",
							name: "Sofia HQ",
							city: "Sofia",
							address: "1 Vitosha Blvd",
							updated_at: "2026-03-07T10:00:00.000Z",
						},
					],
				}),
		});

		const result = (await load({
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result.result.state).toBe("ok");
		expect(result.result.shipments).toHaveLength(1);
		expect(result.offices).toHaveLength(1);
	});

	test("employee shipment create action uses resolved office and redirects to employee detail", async () => {
		const { actions } = await import("../employee/shipments/new/+page.server.ts");
		const fetch = makeFetchMock({
			"POST /shipments": async (request) => {
				const body = await request.json();
				expect(body.current_office_id).toBe("office-sofia");
				expect(body.client_id).toBe("client-acme");
				return jsonResponse({ shipment_id: "SHP-2002" });
			},
		});

		await expectRedirect(
			() =>
				actions.default({
					locals: makeLocals("employee", {
						office_id: "office-sofia",
						current_office_id: "office-sofia",
					}),
					params: { lang: "en" },
					request: makeFormRequest({
						client_id: "client-acme",
						current_office_id: "",
						notes: "Fragile",
					}),
					fetch,
				} as any),
			"/en/app/employee/shipments/SHP-2002",
		);
	});

	test("employee shipments list keeps employee-only role gate", async () => {
		const { load } = await import("../employee/shipments/+page.server.ts");

		await expectHttpError(
			() =>
				load({
					parent: async () => ({ session: makeSession("admin") }),
					fetch: makeFetchMock({}),
					locals: makeLocals("admin"),
				} as any),
			403,
			"error.details.employee_only",
		);
	});

	test("admin audit load returns next-page href from backend pagination", async () => {
		const { load } = await import("../admin/audit/+page.server.ts");
		const fetch = makeFetchMock({
			"GET /admin/audit?limit=10": () =>
				jsonResponse({
					events: [
						{
							id: "audit-1",
							occurred_at: "2026-03-07T10:00:00.000Z",
							actor_user_id: "user-1",
							actor_display_name: "Admin User",
							action_key: "shipment.created",
							entity_type: "shipment",
							entity_id: "SHP-1001",
							entity_label: "Shipment SHP-1001",
							target_route: "/app/admin/shipments/SHP-1001",
							metadata: {},
						},
					],
					page: {
						limit: 10,
						next_cursor: "cursor-2",
						has_next: true,
					},
				}),
		});

		const result = (await load({
			url: new URL("https://app.test/en/app/admin/audit?limit=10"),
			fetch,
			locals: makeLocals("admin"),
		} as any)) as any;

		expect(result.result.state).toBe("ok");
		expect(result.nextPageHref).toBe("/en/app/admin/audit?limit=10&cursor=cursor-2");
	});
});
