import { describe, expect, test } from "bun:test";
import {
	mapListShipmentsResponseDto,
	mapGetShipmentResponseDto,
	mapGetShipmentTimelineResponseDto,
	mapCreateShipmentResponseDto,
	buildStatusHistory,
	deriveCurrentOfficeIdFromTimeline,
	buildStrataPackages,
	mapStatusToBackend,
	buildChangeStatusRequestDto,
} from "../mappers/shipments";
import { HubApiMappingError } from "../normalizers";

describe("shipments mappers", () => {
	// ── List ─────────────────────────────────────────────────────────

	test("valid list payload maps correctly (optional fields)", () => {
		const dto = [
			{
				id: "SHP-0001",
				client_id: "client-acme",
				current_status: "in_transit",
				current_office_id: "office-sofia",
				created_at: "2026-03-01T10:00:00.000Z",
				updated_at: "2026-03-02T12:00:00.000Z",
			},
			{
				id: "SHP-0002",
				client_id: "client-bravo",
				current_status: "delivered",
				current_office_id: null,
				created_at: "2026-03-01T10:00:00.000Z",
				updated_at: "2026-03-02T12:00:00.000Z",
			},
		];

		const items = mapListShipmentsResponseDto(dto as any);
		expect(items).toHaveLength(2);
		expect(items[0]).toMatchObject({
			id: "SHP-0001",
			status: "in_transit",
			office: "office-sofia",
			updatedAt: "2026-03-02T12:00:00.000Z",
		});
		expect(items[1]!.id).toBe("SHP-0002");
		expect(items[1]!.status).toBe("delivered");
		expect(items[1]!.office).toBe("—");
	});

	test("list normalises unknown status to 'unknown'", () => {
		const dto = [
			{
				id: "SHP-0003",
				client_id: "client-acme",
				current_status: "some_weird_status",
				current_office_id: null,
				created_at: "2026-03-01T10:00:00.000Z",
				updated_at: "2026-03-02T12:00:00.000Z",
			},
		];

		const items = mapListShipmentsResponseDto(dto as any);
		expect(items[0]!.status).toBe("unknown");
	});

	test("list normalises status aliases", () => {
		const dto = [
			{
				id: "SHP-0004",
				client_id: "client-acme",
				current_status: "Processed",
				current_office_id: null,
				created_at: "2026-03-01T10:00:00.000Z",
				updated_at: "2026-03-02T12:00:00.000Z",
			},
		];

		const items = mapListShipmentsResponseDto(dto as any);
		expect(items[0]!.status).toBe("pending");
	});

	test("list throws HubApiMappingError when shipments is not an array", () => {
		const bad = { shipments: "not an array" };
		try {
			mapListShipmentsResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /shipments");
			expect(err.field).toBe("shipments");
		}
	});

	test("list throws HubApiMappingError for invalid id type", () => {
		const bad = { shipments: [{ id: 123, current_status: "new" }] };
		try {
			mapListShipmentsResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.field).toContain("shipment.id");
		}
	});

	// ── Detail ───────────────────────────────────────────────────────

	test("valid detail payload maps correctly", () => {
		const dto = {
			id: "SHP-0001",
			client: { id: "client-acme" },
			current_status: "in_transit",
			current_office: { id: "office-sofia" },
			created_at: "2026-03-01T10:00:00.000Z",
			updated_at: "2026-03-02T12:00:00.000Z",
		};

		const detail = mapGetShipmentResponseDto(dto as any);
		expect(detail.id).toBe("SHP-0001");
		expect(detail.client_id).toBe("client-acme");
		expect(detail.current_status).toBe("in_transit");
		expect(detail.current_office_id).toBe("office-sofia");
		expect(detail.created_at).toBe("2026-03-01T10:00:00.000Z");
	});

	test("detail maps null/missing optional fields", () => {
		const dto = {
			id: "SHP-0001",
			client_id: "client-acme",
			current_status: "NEW",
			current_office_id: null,
			created_at: "2026-03-01T10:00:00.000Z",
			updated_at: "2026-03-02T12:00:00.000Z",
		};

		const detail = mapGetShipmentResponseDto(dto as any);
		expect(detail.current_status).toBe("new");
		expect(detail.current_office_id).toBeNull();
		expect(detail.created_at).toBe("2026-03-01T10:00:00.000Z");
		expect(detail.updated_at).toBe("2026-03-02T12:00:00.000Z");
	});

	test("detail throws HubApiMappingError for missing required client_id", () => {
		const bad = { id: "SHP-0001" };
		try {
			mapGetShipmentResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.field).toContain("shipment.client");
		}
	});

	// ── Timeline ─────────────────────────────────────────────────────

	test("valid timeline payload maps correctly", () => {
		const dto = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				scb: "base64scb1",
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				scb: "base64scb2",
			},
		];

		const items = mapGetShipmentTimelineResponseDto(dto as any);
		expect(items).toHaveLength(2);
		expect(items[0]!.seq).toBe(1);
		expect(items[0]!.event_type).toBe("ShipmentCreated");
		expect(items[0]!.scb).toBe("base64scb1");
		expect(items[1]!.scb).toBe("base64scb2");
	});

	test("timeline throws HubApiMappingError for non-number seq", () => {
		const bad = {
			timeline: [
				{
					seq: "not-a-number",
					event_type: "test",
					scb: "base64scb",
				},
			],
		};
		try {
			mapGetShipmentTimelineResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.field).toContain("seq");
		}
	});

	test("timeline throws HubApiMappingError for invalid datetime", () => {
		const bad = { timeline: [{}] };
		try {
			mapGetShipmentTimelineResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.field).toContain("seq");
		}
	});

	// ── Create response ──────────────────────────────────────────────

	test("create response maps correctly", () => {
		const dto = { shipment_id: "SHP-9999" };
		const result = mapCreateShipmentResponseDto(dto as any);
		expect(result.id).toBe("SHP-9999");
	});

	test("create response throws for missing shipment.id", () => {
		const bad = { shipment: {} };
		try {
			mapCreateShipmentResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
		}
	});

	// ── buildStatusHistory ───────────────────────────────────────────

	test("buildStatusHistory produces correct rows from timeline", () => {
		const detail = {
			id: "SHP-0001",
			client_id: "c1",
			current_status: "in_transit" as const,
			current_office_id: "office-1",
			created_at: "2026-03-01T10:00:00.000Z",
			updated_at: "2026-03-02T12:00:00.000Z",
		};

		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: {
					status: "NEW",
					actor_user_id: "user-1",
					office_id: "office-1",
					notes: "created",
				},
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				created_at: "2026-03-01T14:00:00.000Z",
				scb: "base64scb2",
				payload: {
					from_status: "NEW",
					to_status: "IN_TRANSIT",
					actor_user_id: "user-2",
					to_office_id: "office-2",
					notes: "moved",
				},
			},
		];

		const history = buildStatusHistory("SHP-0001", detail, timeline);
		expect(history).toHaveLength(2);
		expect(history[0]!.id).toBe("sh-SHP-0001-1");
		expect(history[0]!.from_status).toBeNull();
		expect(history[0]!.to_status).toBe("new");
		expect(history[0]!.actor_user_id).toBe("user-1");
		expect(history[0]!.notes).toBe("created");

		expect(history[1]!.from_status).toBe("new");
		expect(history[1]!.to_status).toBe("in_transit");
		expect(history[1]!.actor_user_id).toBe("user-2");
	});

	test("buildStatusHistory maps tuple payload values directly", () => {
		const detail = {
			id: "SHP-0001",
			client_id: "c1",
			current_status: "accepted" as const,
			current_office_id: "office-fallback",
			created_at: "2026-03-01T10:00:00.000Z",
			updated_at: "2026-03-02T12:00:00.000Z",
		};

		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: [
					"e6006abc-50a0-4506-8497-1d18c605e7a2",
					{
						actor_user_id: "5f334f10-772c-4839-a8ce-8229168d953b",
						event_type: "ShipmentCreated",
						notes: "test",
						occured_at: 1772654576869,
						office_id: "b4c6cb88-2b88-4fda-b4a4-206de2d72c28",
						shipment_id: "e6006abc-50a0-4506-8497-1d18c605e7a2",
						status: "NEW",
					},
				],
			},
		] as unknown as {
			seq: number;
			event_type: string;
			created_at: string;
			scb: string;
			payload: Record<string, unknown> | null;
		}[];

		const history = buildStatusHistory("SHP-0001", detail, timeline);
		expect(history).toHaveLength(1);
		expect(history[0]!.to_status).toBe("new");
		expect(history[0]!.actor_user_id).toBe(
			"5f334f10-772c-4839-a8ce-8229168d953b",
		);
		expect(history[0]!.office_id).toBe("b4c6cb88-2b88-4fda-b4a4-206de2d72c28");
		expect(history[0]!.notes).toBe("test");
		expect(history[0]!.changed_at).toBe("2026-03-04T20:02:56.869Z");
	});

	test("buildStatusHistory inherits last known office when event omits office", () => {
		const detail = {
			id: "SHP-0001",
			client_id: "c1",
			current_status: "pending" as const,
			current_office_id: "office-start",
			created_at: "2026-03-01T10:00:00.000Z",
			updated_at: "2026-03-02T12:00:00.000Z",
		};

		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: {
					status: "NEW",
					office_id: "office-a",
				},
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				created_at: "2026-03-01T11:00:00.000Z",
				scb: "base64scb2",
				payload: {
					from_status: "NEW",
					to_status: "ACCEPTED",
				},
			},
			{
				seq: 3,
				event_type: "StatusChanged",
				created_at: "2026-03-01T12:00:00.000Z",
				scb: "base64scb3",
				payload: {
					from_status: "ACCEPTED",
					to_status: "PENDING",
					office_id: "office-b",
				},
			},
			{
				seq: 4,
				event_type: "StatusChanged",
				created_at: "2026-03-01T13:00:00.000Z",
				scb: "base64scb4",
				payload: {
					from_status: "PENDING",
					to_status: "IN_TRANSIT",
				},
			},
		];

		const history = buildStatusHistory("SHP-0001", detail, timeline);
		expect(history).toHaveLength(4);
		expect(history[0]!.office_id).toBe("office-a");
		expect(history[1]!.office_id).toBe("office-a");
		expect(history[2]!.office_id).toBe("office-b");
		expect(history[3]!.office_id).toBe("office-b");
	});

	test("deriveCurrentOfficeIdFromTimeline uses latest seq office", () => {
		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: { office_id: "office-a" },
			},
			{
				seq: 3,
				event_type: "StatusChanged",
				created_at: "2026-03-01T12:00:00.000Z",
				scb: "base64scb3",
				payload: ["shipment", { office_id: "office-c" }],
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				created_at: "2026-03-01T11:00:00.000Z",
				scb: "base64scb2",
				payload: { office_id: "office-b" },
			},
		] as unknown as {
			seq: number;
			event_type: string;
			created_at: string;
			scb: string;
			payload: Record<string, unknown> | null;
		}[];

		expect(
			deriveCurrentOfficeIdFromTimeline(timeline, "office-fallback"),
		).toBe("office-c");
	});

	test("deriveCurrentOfficeIdFromTimeline falls back when latest has no office", () => {
		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: { office_id: "office-a" },
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				created_at: "2026-03-01T11:00:00.000Z",
				scb: "base64scb2",
				payload: { notes: "no office" },
			},
		];

		expect(deriveCurrentOfficeIdFromTimeline(timeline, "office-fallback")).toBe(
			"office-fallback",
		);
	});

	// ── buildStrataPackages ──────────────────────────────────────────

	test("buildStrataPackages produces chain-linked packages", () => {
		const timeline = [
			{
				seq: 1,
				event_type: "ShipmentCreated",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: null,
			},
			{
				seq: 2,
				event_type: "StatusChanged",
				created_at: "2026-03-01T11:00:00.000Z",
				scb: "base64scb2",
				payload: null,
			},
		];

		const packages = buildStrataPackages("SHP-0001", timeline);
		expect(packages).toHaveLength(2);

		// First package has no prev_hash
		expect(packages[0]!.prev_hash).toBeNull();
		expect(packages[0]!.seq).toBe(1);
		expect(packages[0]!.stream_id).toBe("stream-shipment-SHP-0001");
		expect(packages[0]!.hash).toBeTruthy();

		// Second package links to first
		expect(packages[1]!.prev_hash).toBe(packages[0]!.hash);
		expect(packages[1]!.seq).toBe(2);

		// Hashes are deterministic
		const packages2 = buildStrataPackages("SHP-0001", timeline);
		expect(packages2[0]!.hash).toBe(packages[0]!.hash);
		expect(packages2[1]!.hash).toBe(packages[1]!.hash);
	});

	test("buildStrataPackages keeps decoded payload when timeline provides it", () => {
		const timeline = [
			{
				seq: 1,
				event_type: "StatusChanged",
				created_at: "2026-03-01T10:00:00.000Z",
				scb: "base64scb1",
				payload: {
					from_status: "NEW",
					to_status: "ACCEPTED",
					notes: "decoded",
				},
			},
		];

		const packages = buildStrataPackages("SHP-0001", timeline);
		expect(packages).toHaveLength(1);
		expect(packages[0]!.payload_json).toEqual({
			from_status: "NEW",
			to_status: "ACCEPTED",
			notes: "decoded",
		});
	});

	// ── mapStatusToBackend ──────────────────────────────────────────

	test("mapStatusToBackend maps all known frontend statuses", () => {
		expect(mapStatusToBackend("new")).toBe("NEW");
		expect(mapStatusToBackend("accepted")).toBe("ACCEPTED");
		expect(mapStatusToBackend("pending")).toBe("PROCESSED");
		expect(mapStatusToBackend("processed")).toBe("PROCESSED");
		expect(mapStatusToBackend("in_transit")).toBe("IN_TRANSIT");
		expect(mapStatusToBackend("delivered")).toBe("DELIVERED");
		expect(mapStatusToBackend("cancelled")).toBe("CANCELLED");
	});

	test("mapStatusToBackend is case-insensitive", () => {
		expect(mapStatusToBackend("NEW")).toBe("NEW");
		expect(mapStatusToBackend("Accepted")).toBe("ACCEPTED");
		expect(mapStatusToBackend("In_Transit")).toBe("IN_TRANSIT");
		expect(mapStatusToBackend("PENDING")).toBe("PROCESSED");
	});

	test("mapStatusToBackend throws HubApiMappingError for unknown status", () => {
		try {
			mapStatusToBackend("bogus_status");
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("POST /shipments/:id/status");
			expect(err.field).toBe("to_status");
		}
	});

	// ── buildChangeStatusRequestDto ─────────────────────────────────

	test("buildChangeStatusRequestDto builds DTO with all fields", () => {
		const dto = buildChangeStatusRequestDto({
			to_status: "in_transit",
			to_office_id: "office-plovdiv",
			notes: "Moving to next office",
		});

		expect(dto.to_status).toBe("IN_TRANSIT");
		expect(dto.to_office_id).toBe("office-plovdiv");
		expect(dto.notes).toBe("Moving to next office");
	});

	test("buildChangeStatusRequestDto with status only (nulls for optional)", () => {
		const dto = buildChangeStatusRequestDto({
			to_status: "accepted",
			to_office_id: null,
			notes: null,
		});

		expect(dto.to_status).toBe("ACCEPTED");
		expect(dto.to_office_id).toBeNull();
		expect(dto.notes).toBeNull();
	});

	test("buildChangeStatusRequestDto with omitted optional fields", () => {
		const dto = buildChangeStatusRequestDto({
			to_status: "cancelled",
		});

		expect(dto.to_status).toBe("CANCELLED");
		expect(dto.to_office_id).toBeNull();
		expect(dto.notes).toBeNull();
	});

	test("buildChangeStatusRequestDto maps pending alias to PROCESSED", () => {
		const dto = buildChangeStatusRequestDto({
			to_status: "pending",
			to_office_id: null,
			notes: "Processing shipment",
		});

		expect(dto.to_status).toBe("PROCESSED");
		expect(dto.notes).toBe("Processing shipment");
	});

	test("buildChangeStatusRequestDto throws for empty to_status", () => {
		try {
			buildChangeStatusRequestDto({
				to_status: "",
			});
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("POST /shipments/:id/status");
			expect(err.field).toBe("to_status");
		}
	});

	test("buildChangeStatusRequestDto trims whitespace-only notes to null", () => {
		const dto = buildChangeStatusRequestDto({
			to_status: "delivered",
			notes: "   ",
		});

		expect(dto.to_status).toBe("DELIVERED");
		// cleanNullableString trims whitespace-only to null
		expect(dto.notes).toBeNull();
	});
});
