export type TimeseriesPointDto = {
	bucket_start: string;
	value: number;
};

export type AdminOverviewResponseDto = {
	total_shipments: number;
	shipments_vs_last_period: number;
	shipments_timeseries: TimeseriesPointDto[];
	total_clients: number;
	clients_vs_last_period: number;
	clients_timeseries: TimeseriesPointDto[];
	total_offices: number;
	offices_vs_last_period: number;
	offices_timeseries: TimeseriesPointDto[];
	total_employees: number;
	assigned_employees: number;
	unassigned_employees: number;
	employees_timeseries: TimeseriesPointDto[];
};

export type EmployeeOverviewResponseDto = {
	active_shipments: number;
	active_vs_last_period: number;
	active_timeseries: TimeseriesPointDto[];
	pending_shipments: number;
	pending_vs_last_period: number;
	pending_timeseries: TimeseriesPointDto[];
	deliveries_today: number;
	deliveries_vs_last_period: number;
	deliveries_timeseries: TimeseriesPointDto[];
};
