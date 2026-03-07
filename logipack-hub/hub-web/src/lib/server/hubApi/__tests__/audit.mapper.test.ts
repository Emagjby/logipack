import { describe, expect, test } from "bun:test";
import { mapListAuditResponseDto } from "../mappers/audit";
import { HubApiMappingError } from "../normalizers";

describe("audit mapper", () => {
	test("maps valid audit list payload", () => {
		const result = mapListAuditResponseDto({
			events: [
				{
					id: "audit-1",
					occurred_at: "2026-03-07T10:00:00.000Z",
					actor_user_id: "user-1",
					actor_display_name: "Admin User",
					action_key: "shipment.created",
					entity_type: "shipment",
					entity_id: "shipment-1",
					entity_label: "Shipment shipment-1",
					target_route: "/app/admin/shipments/shipment-1",
					metadata: {
						client_id: "client-1",
					},
				},
			],
			page: {
				limit: 10,
				next_cursor: "next-cursor",
				has_next: true,
			},
		} as any);

		expect(result.events).toHaveLength(1);
		expect(result.events[0]?.action_key).toBe("shipment.created");
		expect(result.page.next_cursor).toBe("next-cursor");
		expect(result.page.has_next).toBe(true);
	});

	test("throws with context when required fields are invalid", () => {
		try {
			mapListAuditResponseDto({
				events: [{ id: 1, occurred_at: "2026-03-07T10:00:00.000Z" }],
				page: { limit: 10, has_next: false, next_cursor: null },
			} as any);
			throw new Error("expected mapper to throw");
		} catch (error) {
			expect(error).toBeInstanceOf(HubApiMappingError);
			const err = error as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/audit");
			expect(err.field).toContain("event.id");
		}
	});
});
