import { describe, expect, test } from "bun:test";
import {
	mapAdminOverviewResponseDto,
	mapEmployeeOverviewResponseDto,
} from "../mappers/analytics";
import { HubApiMappingError } from "../normalizers";

describe("analytics mapper", () => {
	test("maps valid admin overview payload", () => {
		const result = mapAdminOverviewResponseDto({
			total_shipments: 12,
			shipments_vs_last_period: 3,
			shipments_timeseries: [{ bucket_start: "2026-03-10", value: 4 }],
			total_clients: 8,
			clients_vs_last_period: 1,
			clients_timeseries: [{ bucket_start: "2026-03-10", value: 2 }],
			total_offices: 4,
			offices_vs_last_period: 0,
			offices_timeseries: [{ bucket_start: "2026-03-10", value: 1 }],
			total_employees: 7,
			assigned_employees: 5,
			unassigned_employees: 2,
			employees_timeseries: [{ bucket_start: "2026-03-10", value: 2 }],
		} as any);

		expect(result.total_shipments).toBe(12);
		expect(result.shipments_timeseries[0]).toEqual({
			bucket_start: "2026-03-10",
			value: 4,
		});
		expect(result.employees_timeseries[0]?.value).toBe(2);
	});

	test("maps valid employee overview payload", () => {
		const result = mapEmployeeOverviewResponseDto({
			active_shipments: 6,
			active_vs_last_period: 2,
			active_timeseries: [{ bucket_start: "2026-03-10", value: 6 }],
			pending_shipments: 3,
			pending_vs_last_period: -1,
			pending_timeseries: [{ bucket_start: "2026-03-10", value: 3 }],
			deliveries_today: 2,
			deliveries_vs_last_period: 1,
			deliveries_timeseries: [{ bucket_start: "2026-03-10", value: 2 }],
		} as any);

		expect(result.pending_vs_last_period).toBe(-1);
		expect(result.pending_timeseries[0]?.value).toBe(3);
		expect(result.deliveries_timeseries[0]?.value).toBe(2);
	});

	test("throws with field context when a metric is invalid", () => {
		try {
			mapAdminOverviewResponseDto({
				total_shipments: "12",
				shipments_vs_last_period: 3,
				shipments_timeseries: [],
				total_clients: 8,
				clients_vs_last_period: 1,
				clients_timeseries: [],
				total_offices: 4,
				offices_vs_last_period: 0,
				offices_timeseries: [],
				total_employees: 7,
				assigned_employees: 5,
				unassigned_employees: 2,
				employees_timeseries: [],
			} as any);
			throw new Error("expected mapper to throw");
		} catch (error) {
			expect(error).toBeInstanceOf(HubApiMappingError);
			const err = error as HubApiMappingError;
			expect(err.endpoint).toBe("GET /analytics/admin/overview");
			expect(err.field).toBe("total_shipments");
		}
	});
});
