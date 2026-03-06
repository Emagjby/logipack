import { describe, expect, test } from "bun:test";
import { createHubApiClient } from "../httpClient";
import { HubApiError } from "../errors";
import {
	listShipments,
	getShipment,
	getShipmentTimeline,
	createShipment,
	changeShipmentStatus,
} from "../services/shipments";

function makeFetchMock(
	fn: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
	return fn as unknown as typeof fetch;
}

describe("shipments service", () => {
	test("listShipments success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify([
					{
						id: "SHP-0001",
						client_id: "client-acme",
						current_status: "in_transit",
						current_office_id: "office-sofia",
						created_at: "2026-03-01T10:00:00.000Z",
						updated_at: "2026-03-02T12:00:00.000Z",
					},
				]),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const shipments = await listShipments(client);
		expect(shipments).toHaveLength(1);
		expect(shipments[0]!.id).toBe("SHP-0001");
		expect(shipments[0]!.status).toBe("in_transit");
	});

	test("getShipment success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments/SHP-0001");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify({
					id: "SHP-0001",
					client: { id: "client-acme" },
					current_status: "in_transit",
					current_office: { id: "office-sofia" },
					created_at: "2026-03-01T10:00:00.000Z",
					updated_at: "2026-03-02T12:00:00.000Z",
				}),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const detail = await getShipment(client, "SHP-0001");
		expect(detail.id).toBe("SHP-0001");
		expect(detail.client_id).toBe("client-acme");
		expect(detail.current_status).toBe("in_transit");
	});

	test("getShipmentTimeline success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments/SHP-0001/timeline");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify([
					{
						seq: 1,
						event_type: "ShipmentCreated",
						scb: "base64scb1",
					},
					{
						seq: 2,
						event_type: "StatusChanged",
						scb: "base64scb2",
						payload: {
							from_status: "NEW",
							to_status: "ACCEPTED",
							actor_user_id: "user-1",
							to_office_id: "office-sofia",
							notes: "ok",
						},
					},
				]),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const timeline = await getShipmentTimeline(client, "SHP-0001");
		expect(timeline).toHaveLength(2);
		expect(timeline[0]!.seq).toBe(1);
		expect(timeline[0]!.event_type).toBe("ShipmentCreated");
	});

	test("getShipmentTimeline forwards query params", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments/SHP-0001/timeline?format=PRETTY");
			expect(init?.method).toBe("GET");

			return new Response(
				JSON.stringify([
					{
						seq: 1,
						event_type: "ShipmentCreated",
						scb: "base64scb1",
					},
				]),
				{ status: 200 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const timeline = await getShipmentTimeline(
			client,
			"SHP-0001",
			new URLSearchParams({ format: "PRETTY" }),
		);
		expect(timeline).toHaveLength(1);
		expect(timeline[0]!.seq).toBe(1);
	});

	test("createShipment success", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments");
			expect(init?.method).toBe("POST");

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");

			return new Response(
				JSON.stringify({
					shipment_id: "SHP-9999",
				}),
				{ status: 201 },
			);
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		const created = await createShipment(client, {
			client_id: "client-acme",
			current_office_id: "office-sofia",
			notes: null,
		});
		expect(created.id).toBe("SHP-9999");
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

		await expect(getShipment(client, "missing")).rejects.toBeInstanceOf(
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
			createShipment(client, {
				client_id: "",
				current_office_id: null,
				notes: null,
			}),
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

		await expect(listShipments(client)).rejects.toBeInstanceOf(HubApiError);
	});

	test("changeShipmentStatus success (204 No Content)", async () => {
		const fetchMock = makeFetchMock(async (url, init) => {
			expect(String(url)).toContain("/shipments/SHP-0001/status");
			expect(init?.method).toBe("POST");

			const headers = init!.headers as Record<string, string>;
			expect(headers["Content-Type"]).toBe("application/json");

			const body = JSON.parse(init!.body as string);
			expect(body.to_status).toBe("IN_TRANSIT");
			expect(body.to_office_id).toBe("office-plovdiv");
			expect(body.notes).toBe("Shipping out");

			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		// Should resolve without throwing (void return)
		await changeShipmentStatus(client, "SHP-0001", {
			to_status: "in_transit",
			to_office_id: "office-plovdiv",
			notes: "Shipping out",
		});
	});

	test("changeShipmentStatus with minimal input (no office/notes)", async () => {
		const fetchMock = makeFetchMock(async (_url, init) => {
			const body = JSON.parse(init!.body as string);
			expect(body.to_status).toBe("ACCEPTED");
			expect(body.to_office_id).toBeNull();
			expect(body.notes).toBeNull();

			return new Response(null, { status: 204 });
		});

		const client = createHubApiClient({
			fetch: fetchMock,
			locals: { session: { access_token: "tok" } } as any,
			baseUrl: "https://example.com",
		});

		await changeShipmentStatus(client, "SHP-0002", {
			to_status: "accepted",
		});
	});

	test("changeShipmentStatus error propagation: 409 -> HubApiError", async () => {
		const fetchMock = makeFetchMock(async () => {
			return new Response(
				JSON.stringify({ code: "INVALID_TRANSITION", message: "Cannot go from NEW to DELIVERED" }),
				{
					status: 409,
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
			changeShipmentStatus(client, "SHP-0001", {
				to_status: "delivered",
			}),
		).rejects.toBeInstanceOf(HubApiError);
	});
});
