import { describe, expect, test } from "bun:test";
import { mapReportResponseDto } from "../mappers/reports";
import { HubApiMappingError } from "../normalizers";

describe("reports mapper", () => {
	test("maps valid tabular report payload", () => {
		const result = mapReportResponseDto(
			{
				report_name: "shipments-by-status",
				generated_at: "2026-03-10T09:00:00.000Z",
				columns: ["status", "shipment_count"],
				rows: [
					["DELIVERED", 3],
					["NEW", 5],
				],
			} as any,
			"shipments-by-status",
		);

		expect(result.report_name).toBe("shipments-by-status");
		expect(result.columns).toEqual(["status", "shipment_count"]);
		expect(result.rows[1]).toEqual(["NEW", 5]);
	});

	test("throws with field context when a row cell is invalid", () => {
		try {
			mapReportResponseDto(
				{
					report_name: "shipments-by-status",
					generated_at: "2026-03-10T09:00:00.000Z",
					columns: ["status"],
					rows: [[{ bad: true }]],
				} as any,
				"shipments-by-status",
			);
			throw new Error("expected mapper to throw");
		} catch (error) {
			expect(error).toBeInstanceOf(HubApiMappingError);
			const err = error as HubApiMappingError;
			expect(err.endpoint).toBe("GET /reports/shipments-by-status");
			expect(err.field).toBe("rows[0][0]");
		}
	});
});
