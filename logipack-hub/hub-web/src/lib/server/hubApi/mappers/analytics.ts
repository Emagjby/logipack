import type {
	AdminOverviewResponseDto,
	EmployeeOverviewResponseDto,
	TimeseriesPointDto,
} from "../dto/analytics";
import { HubApiMappingError, requireIsoDateTime, requireRecord } from "../normalizers";

export type TimeseriesPoint = {
	bucket_start: string;
	value: number;
};

export type AdminOverview = {
	total_shipments: number;
	shipments_vs_last_period: number;
	shipments_timeseries: TimeseriesPoint[];
	total_clients: number;
	clients_vs_last_period: number;
	clients_timeseries: TimeseriesPoint[];
	total_offices: number;
	offices_vs_last_period: number;
	offices_timeseries: TimeseriesPoint[];
	total_employees: number;
	assigned_employees: number;
	unassigned_employees: number;
	employees_timeseries: TimeseriesPoint[];
};

export type EmployeeOverview = {
	active_shipments: number;
	active_vs_last_period: number;
	active_timeseries: TimeseriesPoint[];
	pending_shipments: number;
	pending_vs_last_period: number;
	pending_timeseries: TimeseriesPoint[];
	deliveries_today: number;
	deliveries_vs_last_period: number;
	deliveries_timeseries: TimeseriesPoint[];
};

function requireNumber(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): number {
	if (typeof args.value !== "number" || !Number.isFinite(args.value)) {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected finite number for ${args.field}`,
		});
	}
	return args.value;
}

function mapTimeseriesPointDto(
	dto: TimeseriesPointDto,
	endpoint: string,
	field: string,
): TimeseriesPoint {
	const obj = requireRecord({ endpoint, field, value: dto });
	return {
		bucket_start: requireIsoDateTimeOrDate({
			endpoint,
			field: `${field}.bucket_start`,
			value: obj.bucket_start,
		}),
		value: requireNumber({
			endpoint,
			field: `${field}.value`,
			value: obj.value,
		}),
	};
}

function requireTimeseries(
	endpoint: string,
	field: string,
	value: unknown,
): TimeseriesPoint[] {
	if (!Array.isArray(value)) {
		throw new HubApiMappingError({
			endpoint,
			field,
			message: `expected ${field}[]`,
		});
	}

	return value.map((item, index) =>
		mapTimeseriesPointDto(item as TimeseriesPointDto, endpoint, `${field}[${index}]`),
	);
}

function requireIsoDateTimeOrDate(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): string {
	if (typeof args.value !== "string") {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected string for ${args.field}`,
		});
	}

	const trimmed = args.value.trim();
	if (/^\d{4}-\d{2}-\d{2}$/.test(trimmed)) {
		return trimmed;
	}

	return requireIsoDateTime(args);
}

export function mapAdminOverviewResponseDto(
	dto: AdminOverviewResponseDto,
): AdminOverview {
	const endpoint = "GET /analytics/admin/overview";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return {
		total_shipments: requireNumber({
			endpoint,
			field: "total_shipments",
			value: obj.total_shipments,
		}),
		shipments_vs_last_period: requireNumber({
			endpoint,
			field: "shipments_vs_last_period",
			value: obj.shipments_vs_last_period,
		}),
		shipments_timeseries: requireTimeseries(
			endpoint,
			"shipments_timeseries",
			obj.shipments_timeseries,
		),
		total_clients: requireNumber({
			endpoint,
			field: "total_clients",
			value: obj.total_clients,
		}),
		clients_vs_last_period: requireNumber({
			endpoint,
			field: "clients_vs_last_period",
			value: obj.clients_vs_last_period,
		}),
		clients_timeseries: requireTimeseries(
			endpoint,
			"clients_timeseries",
			obj.clients_timeseries,
		),
		total_offices: requireNumber({
			endpoint,
			field: "total_offices",
			value: obj.total_offices,
		}),
		offices_vs_last_period: requireNumber({
			endpoint,
			field: "offices_vs_last_period",
			value: obj.offices_vs_last_period,
		}),
		offices_timeseries: requireTimeseries(
			endpoint,
			"offices_timeseries",
			obj.offices_timeseries,
		),
		total_employees: requireNumber({
			endpoint,
			field: "total_employees",
			value: obj.total_employees,
		}),
		assigned_employees: requireNumber({
			endpoint,
			field: "assigned_employees",
			value: obj.assigned_employees,
		}),
		unassigned_employees: requireNumber({
			endpoint,
			field: "unassigned_employees",
			value: obj.unassigned_employees,
		}),
		employees_timeseries: requireTimeseries(
			endpoint,
			"employees_timeseries",
			obj.employees_timeseries,
		),
	};
}

export function mapEmployeeOverviewResponseDto(
	dto: EmployeeOverviewResponseDto,
): EmployeeOverview {
	const endpoint = "GET /analytics/employee/overview";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return {
		active_shipments: requireNumber({
			endpoint,
			field: "active_shipments",
			value: obj.active_shipments,
		}),
		active_vs_last_period: requireNumber({
			endpoint,
			field: "active_vs_last_period",
			value: obj.active_vs_last_period,
		}),
		active_timeseries: requireTimeseries(
			endpoint,
			"active_timeseries",
			obj.active_timeseries,
		),
		pending_shipments: requireNumber({
			endpoint,
			field: "pending_shipments",
			value: obj.pending_shipments,
		}),
		pending_vs_last_period: requireNumber({
			endpoint,
			field: "pending_vs_last_period",
			value: obj.pending_vs_last_period,
		}),
		pending_timeseries: requireTimeseries(
			endpoint,
			"pending_timeseries",
			obj.pending_timeseries,
		),
		deliveries_today: requireNumber({
			endpoint,
			field: "deliveries_today",
			value: obj.deliveries_today,
		}),
		deliveries_vs_last_period: requireNumber({
			endpoint,
			field: "deliveries_vs_last_period",
			value: obj.deliveries_vs_last_period,
		}),
		deliveries_timeseries: requireTimeseries(
			endpoint,
			"deliveries_timeseries",
			obj.deliveries_timeseries,
		),
	};
}
