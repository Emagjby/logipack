import { describe, expect, test } from "bun:test";
import {
	mapGetClientResponseDto,
	mapListClientsResponseDto,
} from "../mappers/clients";
import { HubApiMappingError } from "../normalizers";

describe("clients mappers", () => {
	test("valid list payload maps correctly (optional email/phone)", () => {
		const listDto = {
			clients: [
				{ id: "c1", name: "ACME", email: "a@b.com", phone: null },
				{ id: "c2", name: "No Contact" }, // missing optionals ok
			],
		};

		const items = mapListClientsResponseDto(listDto as any);
		expect(items).toHaveLength(2);
		expect(items[0]).toMatchObject({
			id: "c1",
			name: "ACME",
			email: "a@b.com",
			phone: null,
		});
		expect(items[1]).toMatchObject({ id: "c2", name: "No Contact" });
	});

	test("valid detail payload maps correctly (nullable fields)", () => {
		const detailDto = {
			client: {
				id: "c1",
				name: "ACME",
				email: "contact@acme.com",
				phone: "123",
				created_at: "2026-03-02T12:00:00.000Z",
				updated_at: "2026-03-02T12:30:00.000Z",
				deleted_at: null,
			},
		};

		const detail = mapGetClientResponseDto(detailDto as any);
		expect(detail.id).toBe("c1");
		expect(detail.deleted_at).toBeNull();
		expect(detail.created_at).toBe("2026-03-02T12:00:00.000Z");
	});

	test("invalid required field types throw HubApiMappingError with context", () => {
		const bad = { clients: [{ id: 123, name: "X" }] };

		try {
			mapListClientsResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/clients");
			expect(err.field).toContain("client.id");
		}
	});

	test("nullable email/phone stable when present as whitespace", () => {
		const detailDto = {
			client: {
				id: "c1",
				name: "ACME",
				email: "   ",
				phone: "   ",
			},
		};

		const detail = mapGetClientResponseDto(detailDto as any);
		expect(detail.email).toBeNull();
		expect(detail.phone).toBeNull();
	});
});
