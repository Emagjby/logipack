import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import {
	listClients,
	getClient,
	createClient,
	updateClient,
	deleteClient,
} from "../services/clients";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("clients service", () => {
	test("listClients success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/clients");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					clients: [
						{
							id: "c1",
							name: "ACME",
							email: "contact@acme.com",
							phone: null,
							updated_at: "2026-03-02T12:43:00.000Z",
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

		const clients = await listClients(client);
		expect(clients).toHaveLength(1);
		expect(clients[0]!.id).toBe("c1");
	});

	test("getClient success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/clients/c1");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					client: {
						id: "c1",
						name: "ACME",
						email: null,
						phone: "+359888123456",
						updated_at: "2026-03-02T12:43:00.000Z",
						created_at: "2026-03-01T10:00:00.000Z",
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

		const c = await getClient(client, "c1");
		expect(c.id).toBe("c1");
		expect(c.deleted_at).toBeNull();
	});

	test("createClient success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/clients");
			expect(init?.method).toBe("POST");

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");
			expect(init!.body).toBe(
				JSON.stringify({ name: "Nova", email: null, phone: null }),
			);

			return new Response(
				JSON.stringify({
					client: {
						id: "c2",
						name: "Nova",
						email: null,
						phone: null,
						updated_at: "2026-03-02T12:43:00.000Z",
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

		const created = await createClient(client, {
			name: "Nova",
			email: null,
			phone: null,
		});
		expect(created.id).toBe("c2");
		expect(created.name).toBe("Nova");
	});

	test("updateClient success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/clients/c1");
			expect(init?.method).toBe("PUT");

			return new Response(
				JSON.stringify({
					client: {
						id: "c1",
						name: "ACME Updated",
						email: "new@acme.com",
						phone: null,
						updated_at: "2026-03-02T13:43:00.000Z",
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

		const updated = await updateClient(client, "c1", {
			name: "ACME Updated",
			email: "new@acme.com",
			phone: null,
		});
		expect(updated.name).toBe("ACME Updated");
	});

	test("deleteClient success (204)", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/clients/c1");
			expect(init?.method).toBe("DELETE");
			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await deleteClient(client, "c1");
	});

	test("error propagation: 404 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response("", { status: 404 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(getClient(client, "missing")).rejects.toBeInstanceOf(
			HubApiError,
		);
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

		await expect(
			createClient(client, { name: "", email: null, phone: null }),
		).rejects.toBeInstanceOf(HubApiError);
	});

	test("error propagation: 500 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response("boom", { status: 500 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(listClients(client)).rejects.toBeInstanceOf(HubApiError);
	});
});
