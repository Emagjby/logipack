import { describe, expect, test } from "bun:test";
import { HubApiMappingError } from "../normalizers";
import { mapEmployeeOfficesResponseDto } from "../mappers/employeeOffices";

describe("employeeOffices mappers", () => {
	test("maps office assignment context employee optional, currentOffice resolved", () => {
		const dto = {
			employee_id: "emp-1",
			offices: [
				{
					id: "office-2",
					name: "Plovdiv DC",
					city: "Plovdiv",
					address: "Main St",
					updated_at: "2026-03-03T10:00:00.000Z",
				},
			],
			office_ids: ["office-2"],
			employee: {
				id: "emp-1",
				user_id: "user-1",
				user: { id: "user-1", email: "u@x.com", name: "John Doe" },
				offices: null,
			},
		};

		const res = mapEmployeeOfficesResponseDto(dto as any);
		expect(res.state).toBe("ok");
		if (res.state !== "ok") throw new Error("unreachable");

		expect(res.employee.id).toBe("emp-1");
		expect(res.employee.user_display_name).toBe("John Doe");
		expect(res.employee.office_ids).toEqual(["office-2"]);
		expect(res.currentOfficeId).toBe("office-2");
		expect(res.currentOffice?.name).toBe("Plovdiv DC");
		expect(res.hasMultipleOffices).toBe(false);
	});

	test("works when employee hydration is missing (still returns ok)", () => {
		const dto = {
			employee_id: "emp-1",
			offices: [
				{
					id: "office-1",
					name: "Sofia HQ",
					city: "Sofia",
					address: "12 Vitosha Blvd",
					updated_at: "2026-03-03T10:00:00.000Z",
				},
			],
			office_ids: [],
		};

		const res = mapEmployeeOfficesResponseDto(dto as any);
		expect(res.state).toBe("ok");
		if (res.state !== "ok") throw new Error("unreachable");
		expect(res.employee.id).toBe("emp-1");
		expect(res.currentOfficeId).toBeNull();
		expect(res.currentOffice).toBeNull();
	});

	test("office_ids accepts office objects with id field", () => {
		const dto = {
			employee_id: "emp-2",
			offices: [
				{
					id: "office-3",
					name: "Varna Hub",
					city: "Varna",
					address: "Sea 1",
					updated_at: "2026-03-03T10:00:00.000Z",
				},
			],
			office_ids: [{ id: "office-3" }],
		};

		const res = mapEmployeeOfficesResponseDto(dto as any);
		expect(res.state).toBe("ok");
		if (res.state !== "ok") throw new Error("unreachable");
		expect(res.currentOfficeId).toBe("office-3");
	});

	test("invalid office_ids shape throws HubApiMappingError with context", () => {
		const bad = {
			employee_id: "emp-1",
			offices: [],
			office_ids: "office-1",
		};

		try {
			mapEmployeeOfficesResponseDto(bad as any);
			throw new Error("expected mapper to throw");
		} catch (e) {
			expect(e).toBeInstanceOf(HubApiMappingError);
			const err = e as HubApiMappingError;
			expect(err.endpoint).toBe("GET /admin/employees/:id/offices");
			expect(err.field).toBe("office_ids");
		}
	});
});
