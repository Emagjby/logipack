import { describe, expect, test } from "bun:test";
import { mapMeRole } from "../mappers/identity";

describe("mapMeRole", () => {
	test("returns role for valid payload", () => {
		expect(mapMeRole({ role: "admin" })).toBe("admin");
		expect(mapMeRole({ role: "employee" })).toBe("employee");
	});

	test("handles invalid role safely", () => {
		expect(mapMeRole({ role: "user" })).toBe("");
		expect(mapMeRole({ role: "   admin   " })).toBe("admin");
	});
});
