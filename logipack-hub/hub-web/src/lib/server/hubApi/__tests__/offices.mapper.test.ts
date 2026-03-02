import { describe, expect, test } from "bun:test";
import {
	mapListOfficesResponseDto,
	mapGetOfficeResponseDto,
} from "../mappers/offices";
import { HubApiMappingError } from "../normalizers";

describe("offices mappers", () => {
	test("valid office DTO maps correctly (list + detail)", () => {
		const listDto = {
			offices: [
				{
					id: "office-1",
					name: "Sofia Central",
					city: "Sofia",
					address: "1 Vitosha Blvd",
					updated_at: "2026-02-10T10:00:00.000Z",
				},
			],
		};

		const items = mapListOfficesResponseDto(listDto as any);
		expect(items).toHaveLength(1);
		expect(items[0]).toEqual({
			id: "office-1",
			name: "Sofia Central",
			city: "Sofia",
			address: "1 Vitosha Blvd",
			updated_at: "2026-02-10T10:00:00.000Z",
		});

		const detailDto = {
			office: {
				id: "office-1",
				name: "Sofia Central",
				city: "Sofia",
				address: "1 Vitosha Blvd",
				created_at: "2026-01-01T09:00:00.000Z",
				updated_at: "2026-02-10T10:00:00.000Z",
				deleted_at: null,
			},
		};

		const detail = mapGetOfficeResponseDto(detailDto as any);
		expect(detail.id).toBe("office-1");
		expect(detail.created_at).toBe("2026-01-01T09:00:00.000Z");
		expect(detail.deleted_at).toBeNull();
	});

	test("invalid field types throw HubApiMappingError with field context", () => {
		const badListDto = {
			offices: [
				{
					id: 123,
					name: "X",
					city: "Y",
					address: "Z",
					updated_at: "2026-02-10T10:00:00.000Z",
				},
			],
		};

		try {
			mapListOfficesResponseDto(badListDto as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/offices");
			expect(err.field).toContain("id");
		}

		const badDetailDto = {
			office: {
				id: "office-1",
				name: "Sofia Central",
				city: "Sofia",
				address: "1 Vitosha Blvd",
				updated_at: "not-a-date",
			},
		};

		try {
			mapGetOfficeResponseDto(badDetailDto as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/offices/:id");
			expect(err.field).toContain("updated_at");
		}
	});

	test("optional fields are stable when missing (created_at, deleted_at)", () => {
		const detailDtoMissingOptionals = {
			office: {
				id: "office-2",
				name: "Plovdiv",
				city: "Plovdiv",
				address: "2 Main St",
				updated_at: "2026-02-10T10:00:00.000Z",
			},
		};

		const detail = mapGetOfficeResponseDto(detailDtoMissingOptionals as any);
		expect(detail.created_at).toBeUndefined();
		expect(detail.deleted_at).toBeUndefined();
	});
});
