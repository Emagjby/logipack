import type { ReportCellDto, ReportResponseDto } from "../dto/reports";
import {
	HubApiMappingError,
	requireIsoDateTime,
	requireRecord,
	requireString,
} from "../normalizers";

export type ReportCell = string | number | boolean | null;

export type TabularReport = {
	report_name: string;
	generated_at: string;
	columns: string[];
	rows: ReportCell[][];
};

function isReportCell(value: unknown): value is ReportCell {
	return (
		value === null ||
		typeof value === "string" ||
		typeof value === "number" ||
		typeof value === "boolean"
	);
}

export function mapReportResponseDto(
	dto: ReportResponseDto,
	reportName: string,
): TabularReport {
	const endpoint = `GET /reports/${reportName}`;
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.columns)) {
		throw new HubApiMappingError({
			endpoint,
			field: "columns",
			message: "expected columns[]",
		});
	}

	if (!Array.isArray(obj.rows)) {
		throw new HubApiMappingError({
			endpoint,
			field: "rows",
			message: "expected rows[]",
		});
	}

	const columns = obj.columns.map((column, index) =>
		requireString({
			endpoint,
			field: `columns[${index}]`,
			value: column,
			nonEmpty: true,
		}),
	);

	const rows = obj.rows.map((row, rowIndex) => {
		if (!Array.isArray(row)) {
			throw new HubApiMappingError({
				endpoint,
				field: `rows[${rowIndex}]`,
				message: "expected row[]",
			});
		}

		return row.map((cell, cellIndex) => {
			if (!isReportCell(cell)) {
				throw new HubApiMappingError({
					endpoint,
					field: `rows[${rowIndex}][${cellIndex}]`,
					message: "expected report cell value",
				});
			}
			return cell as ReportCellDto;
		});
	});

	return {
		report_name: requireString({
			endpoint,
			field: "report_name",
			value: obj.report_name,
			nonEmpty: true,
		}),
		generated_at: requireIsoDateTime({
			endpoint,
			field: "generated_at",
			value: obj.generated_at,
		}),
		columns,
		rows,
	};
}
