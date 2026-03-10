import { describe, expect, test } from "bun:test";
import { compactId, isIdColumn } from "../idDisplay";

describe("idDisplay", () => {
	test("compactId keeps short ids unchanged and truncates long ids", () => {
		expect(compactId("SHP-1001")).toBe("SHP-1001");
		expect(compactId("6a3c96d5-5f4d-4865-8185-34ae1b2ec78d")).toBe("6a3c96d5...");
	});

	test("isIdColumn matches id columns only", () => {
		expect(isIdColumn("id")).toBe(true);
		expect(isIdColumn("client_id")).toBe(true);
		expect(isIdColumn("shipment_count")).toBe(false);
	});
});
