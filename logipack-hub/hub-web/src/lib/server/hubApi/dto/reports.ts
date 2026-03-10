export type ReportCellDto = string | number | boolean | null;

export type ReportResponseDto = {
	report_name: string;
	generated_at: string;
	columns: string[];
	rows: ReportCellDto[][];
};
