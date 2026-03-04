import { describe, expect, test } from "bun:test";
import {
	mapCreateEmployeeResponseDto,
	mapGetEmployeeResponseDto,
	mapListEmployeesResponseDto,
	mapEmployeeListItemDtoToEmployeeListItem,
	mapEmployeeDtoToEmployeeDetail,
	mapUpdateEmployeeResponseDto,
} from "../mappers/employees";
import { HubApiMappingError } from "../normalizers";

describe("employees mappers", () => {
	test("valid list payload maps correctly with office hydration", () => {
		const dto = {
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
		};

		const out = mapListEmployeesResponseDto(dto as any);

		console.log("Mapped employee list item:", out[0]);

		expect(out[0]).toMatchObject({
			id: "emp-1",
			user_id: "user-1",
			full_name: "Ivan Ivanov",
			email: "ivan@logipack.dev",
			office_id: "office-1",
			office_name: "Sofia HQ",
			office_city: "Sofia",
		});

		expect(out[0]!.user_display_name).toBe("Ivan");
	});

	test("list item mapping omits office fields when offices are missing", () => {
		const item = {
			id: "emp-2",
			user_id: "user-2",
			full_name: "No Office",
			email: "no-office@logipack.dev",
		};

		const out = mapEmployeeListItemDtoToEmployeeListItem(item as any);

		expect(out).toMatchObject({
			id: "emp-2",
			user_id: "user-2",
			full_name: "No Office",
			email: "no-office@logipack.dev",
		});

		expect("office_id" in out).toBe(false);
		expect("office_name" in out).toBe(false);
		expect("office_city" in out).toBe(false);
	});

	test("list item mapping omits office fields when offices are empty", () => {
		const item = {
			id: "emp-2b",
			user_id: "user-2b",
			full_name: "Empty Office",
			email: "empty-office@logipack.dev",
			offices: [],
		};

		const out = mapEmployeeListItemDtoToEmployeeListItem(item as any);

		expect(out).toMatchObject({
			id: "emp-2b",
			user_id: "user-2b",
			full_name: "Empty Office",
			email: "empty-office@logipack.dev",
		});

		expect("office_id" in out).toBe(false);
		expect("office_name" in out).toBe(false);
		expect("office_city" in out).toBe(false);
	});

	test("valid detail payload maps correctly timestamps, nullable deleted_at", () => {
		const dto = {
			employee: {
				id: "emp-3",
				user_id: "user-3",
				full_name: "Detail Guy",
				email: "detail@logipack.dev",
				user_display_name: null,
				offices: [
					{
						id: "office-9",
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
		};

		const out = mapGetEmployeeResponseDto(dto as any);

		console.log("Mapped employee detail:", out);

		expect(out).toMatchObject({
			id: "emp-3",
			user_id: "user-3",
			full_name: "Detail Guy",
			email: "detail@logipack.dev",
			user_display_name: null,
			office_id: "office-9",
			office_name: "Varna Port",
			office_city: "Varna",
			created_at: "2026-03-01T09:00:00.000Z",
			updated_at: "2026-03-02T09:00:00.000Z",
		});

		expect(out.deleted_at).toBeNull();
	});

	test("detail mapping keeps optional timestamps stable when missing", () => {
		const dto = {
			employee: {
				id: "emp-4",
				user_id: "user-4",
				full_name: "No Times",
				email: "times@logipack.dev",
			},
		};

		const out = mapGetEmployeeResponseDto(dto as any);

		expect(out.created_at).toBeUndefined();
		expect(out.updated_at).toBeUndefined();
		expect(out.deleted_at).toBeUndefined();
	});

	test("malformed list payload throws HubApiMappingError with endpoint+field context", () => {
		const bad = { employees: "nope" };

		try {
			mapListEmployeesResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/employees");
			expect(err.field).toBe("employees");
		}
	});

	test("invalid field types throw HubApiMappingError with deterministic context", () => {
		const badList = {
			employees: [
				{
					id: 123,
					user_id: "user-x",
					full_name: "X",
					email: "x@x.com",
				},
			],
		};

		try {
			mapListEmployeesResponseDto(badList as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/employees");
			expect(err.field).toContain("employee.id");
		}
	});

	test("whitespace deleted_at normalizes to null regression", () => {
		const dto = {
			employee: {
				id: "emp-5",
				user_id: "user-5",
				full_name: "Whitespace",
				email: "w@logipack.dev",
				deleted_at: "   ",
			},
		};

		const out = mapEmployeeDtoToEmployeeDetail(dto.employee as any);
		expect(out.deleted_at).toBeNull();
	});

	test("create response maps to employee detail", () => {
		const dto = {
			employee: {
				id: "emp-20",
				user_id: "user-20",
				full_name: "Created",
				email: "created@logipack.dev",
				user_display_name: null,
				created_at: "2026-03-01T09:00:00.000Z",
				updated_at: "2026-03-01T09:00:00.000Z",
				deleted_at: null,
			},
		};

		const out = mapCreateEmployeeResponseDto(dto as any);
		expect(out.id).toBe("emp-20");
		expect(out.email).toBe("created@logipack.dev");
	});

	test("update response maps to employee detail", () => {
		const dto = {
			employee: {
				id: "emp-21",
				user_id: "user-21",
				full_name: "Updated",
				email: "updated@logipack.dev",
				user_display_name: null,
				created_at: "2026-03-01T09:00:00.000Z",
				updated_at: "2026-03-02T09:00:00.000Z",
				deleted_at: null,
			},
		};

		const out = mapUpdateEmployeeResponseDto(dto as any);
		expect(out.id).toBe("emp-21");
		expect(out.email).toBe("updated@logipack.dev");
	});
});
