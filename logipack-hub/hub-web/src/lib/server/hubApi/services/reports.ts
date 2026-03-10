import type { HubApiClient } from "../index";
import type { ReportResponseDto } from "../dto/reports";
import type { TabularReport } from "../mappers/reports";
import { mapReportResponseDto } from "../mappers/reports";

export type ReportName =
	| "shipments-by-status"
	| "shipments-by-office"
	| "shipments-by-client"
	| "shipments-by-period";

export type ReportBucket = "day" | "week" | "month";

export async function getReport(
	client: HubApiClient,
	reportName: ReportName,
	args?: {
		from?: string | null;
		to?: string | null;
		bucket?: ReportBucket | null;
	},
	timeoutMs = 10_000,
): Promise<TabularReport> {
	const query = new URLSearchParams();

	if (args?.from) {
		query.set("from", args.from);
	}

	if (args?.to) {
		query.set("to", args.to);
	}

	if (args?.bucket && reportName === "shipments-by-period") {
		query.set("bucket", args.bucket);
	}

	const suffix = query.size > 0 ? `?${query.toString()}` : "";
	const res = await client.get<ReportResponseDto>(`/reports/${reportName}${suffix}`, {
		timeoutMs,
	});

	return mapReportResponseDto(res.data, reportName);
}
