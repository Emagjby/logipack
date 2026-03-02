import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import {
	createOffice,
	getOffice,
	listOffices,
	updateOffice,
	deleteOffice,
} from "../services/offices";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("offices service", () => {
	test("listOffices success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/offices");
			expect(init?.method).toBe("GET");
			return new Response(
				JSON.stringify({
					offices: [
						{
							id: "o1",
							name: "Sofia HQ",
							city: "Sofia",
							address: "12 Vitosha Blvd",
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

		const offices = await listOffices(client);
		expect(offices).toHaveLength(1);
		expect(offices[0]!.id).toBe("o1");
	});

	test("getOffice success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/offices/o1");
			expect(init?.method).toBe("GET");
			return new Response(
				JSON.stringify({
					office: {
						id: "o1",
						name: "Sofia HQ",
						city: "Sofia",
						address: "12 Vitosha Blvd",
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

		const office = await getOffice(client, "o1");
		expect(office.id).toBe("o1");
		expect(office.deleted_at).toBeNull();
	});

	test("createOffice success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/offices");
			expect(init?.method).toBe("POST");

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");

			expect(init!.body).toBe(
				JSON.stringify({ name: "X", city: "Y", address: "Z" }),
			);

			return new Response(
				JSON.stringify({
					office: {
						id: "o2",
						name: "X",
						city: "Y",
						address: "Z",
						updated_at: "2026-03-02T12:43:00.000Z",
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

		const created = await createOffice(client, {
			name: "X",
			city: "Y",
			address: "Z",
		});
		expect(created.id).toBe("o2");
	});

	test("updateOffice success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/offices/o1");
			expect(init?.method).toBe("PUT");

			return new Response(
				JSON.stringify({
					office: {
						id: "o1",
						name: "new",
						city: "sofia",
						address: "addr",
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

		const updated = await updateOffice(client, "o1", {
			name: "new",
			city: "sofia",
			address: "addr",
		});
		expect(updated.name).toBe("new");
	});

	test("deleteOffice success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/admin/offices/o1");
			expect(init?.method).toBe("DELETE");

			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(getOffice(client, "missing")).rejects.toBeInstanceOf(
			HubApiError,
		);
	});

	test("error propagation uses HubApiError 404", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response("", { status: 404 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await expect(getOffice(client, "missing")).rejects.toBeInstanceOf(
			HubApiError,
		);
	});
});
