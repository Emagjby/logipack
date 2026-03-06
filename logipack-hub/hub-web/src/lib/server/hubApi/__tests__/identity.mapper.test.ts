import { describe, expect, test } from "bun:test";
import { mapMeContext, mapMeRole } from "../mappers/identity";

describe("mapMeRole", () => {
	test("returns role for valid payload", () => {
		expect(mapMeRole({ role: "admin" })).toBe("admin");
		expect(mapMeRole({ role: "employee" })).toBe("employee");
	});

	test("handles invalid role safely", () => {
		expect(mapMeRole({ role: "user" })).toBe("");
		expect(mapMeRole({ role: "   admin   " })).toBe("admin");
	});

	test("maps me context with office data", () => {
		expect(
			mapMeContext({
				role: "employee",
				office_ids: [" o1 ", "", "o2"],
				current_office_id: " o2 ",
			}),
		).toEqual({ role: "employee", office_ids: ["o1", "o2"], current_office_id: "o2" });
	});
});
