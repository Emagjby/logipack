import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	getReport,
	type ReportBucket,
	type ReportName,
	HubApiError,
} from "$lib/server/hubApi";
import type { PageServerLoad } from "./$types";

type ReportResult =
	| {
			state: "ok";
			report: Awaited<ReturnType<typeof getReport>>;
	  }
	| {
			state: "error";
			report: null;
			message: string;
	  };

function parseReportName(raw: string | null): ReportName {
	switch (raw) {
		case "shipments-by-office":
		case "shipments-by-client":
		case "shipments-by-period":
			return raw;
		case "shipments-by-status":
		default:
			return "shipments-by-status";
	}
}

function parseBucket(raw: string | null): ReportBucket {
	switch (raw) {
		case "week":
		case "month":
			return raw;
		case "day":
		default:
			return "day";
	}
}

function cleanDateParam(raw: string | null): string | null {
	const trimmed = raw?.trim() ?? "";
	return trimmed.length > 0 ? trimmed : null;
}

export const load: PageServerLoad = async ({ url, fetch, locals }) => {
	const selectedReport = parseReportName(url.searchParams.get("report"));
	const bucket = parseBucket(url.searchParams.get("bucket"));
	const from = cleanDateParam(url.searchParams.get("from"));
	const to = cleanDateParam(url.searchParams.get("to"));

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const report = await getReport(client, selectedReport, {
			from,
			to,
			bucket,
		});

		const result: ReportResult = {
			state: "ok",
			report,
		};

		return {
			selectedReport,
			selectedBucket: bucket,
			from,
			to,
			result,
		};
	} catch (error) {
		if (error instanceof HubApiError) {
			console.error('admin.reports.load failed', {
				status: error.status,
				code: error.code,
				message: error.message,
				upstream: error.upstream,
			});
		} else {
			console.error('admin.reports.load failed', error);
		}

		return {
			selectedReport,
			selectedBucket: bucket,
			from,
			to,
			result: {
				state: "error" as const,
				report: null,
				message: "reports.error.generic",
			},
		};
	}
};
