import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	getEmployeeOverview,
	type AnalyticsSpan,
	type ShipmentListItem,
	HubApiError,
	listShipments,
} from "$lib/server/hubApi";
import { resolveEmployeeOffice } from "$lib/server/employeeOffice";
import { normalizeShipmentStatus, statusLabelKey } from "$lib/domain/shipmentStatus";
import type { PageServerLoad } from "./$types";
import { _ } from "svelte-i18n";
import { get } from "svelte/store";

type TranslationValues = Record<
	string,
	string | number | boolean | Date | null | undefined
>;

function f(key: string, vars?: TranslationValues): string {
	return get(_)(key, vars ? { values: vars } : undefined);
}

function parseSpan(raw: string | null): AnalyticsSpan {
	return raw === "30d" || raw === "90d" ? raw : "7d";
}

function periodLabel(span: AnalyticsSpan): string {
	return f(`analytics.period.${span}`);
}

function signedDelta(value: number): string {
	return value > 0 ? `+${value}` : `${value}`;
}

function dashboardStatus(
	status: ShipmentListItem["status"],
): "pending" | "in-transit" | "delivered" | "cancelled" {
	switch (status) {
		case "new":
		case "accepted":
		case "pending":
			return "pending";
		case "in_transit":
			return "in-transit";
		case "delivered":
			return "delivered";
		case "cancelled":
		default:
			return "cancelled";
	}
}

function formatDashboardDate(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return "—";
	return new Intl.DateTimeFormat("en-GB", {
		month: "short",
		day: "numeric",
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
		timeZone: "UTC",
	}).format(date);
}

function activityGroupLabel(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return f("recent");

	const now = new Date();
	const today = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
	const target = Date.UTC(
		date.getUTCFullYear(),
		date.getUTCMonth(),
		date.getUTCDate(),
	);
	const diffDays = Math.round((today - target) / 86_400_000);

	if (diffDays <= 0) return f("today");
	if (diffDays === 1) return f("yesterday");
	return new Intl.DateTimeFormat("en-GB", {
		month: "short",
		day: "numeric",
		timeZone: "UTC",
	}).format(date);
}

function formatActivityTime(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return "—";
	return new Intl.DateTimeFormat("en-GB", {
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
		timeZone: "UTC",
	}).format(date);
}

function buildActivity(shipments: ShipmentListItem[]) {
	const groups = new Map<
		string,
		{
			group: string;
			items: {
				time: string;
				shipmentId: string;
				title: string;
				tag: string;
			}[];
		}
	>();

	for (const shipment of shipments.slice(0, 8)) {
		const group = activityGroupLabel(shipment.updatedAt);
		const normalized = normalizeShipmentStatus(shipment.status);
		const statusLabel = f(statusLabelKey(normalized));
		const bucket = groups.get(group) ?? { group, items: [] };
		bucket.items.push({
			time: formatActivityTime(shipment.updatedAt),
			shipmentId: shipment.id,
			title: `- ${statusLabel} · ${shipment.office}`,
			tag: dashboardStatus(shipment.status),
		});
		groups.set(group, bucket);
	}

	return Array.from(groups.values());
}

function getGreeting(name: string): string {
	const h = new Date().getHours();
	const firstName = name.split(" ")[0];
	if (h < 12) return `${f("greet.morning")}, ${firstName}`;
	if (h < 18) return `${f("greet.afternoon")}, ${firstName}`;
	return `${f("greet.evening")}, ${firstName}`;
}

export const load: PageServerLoad = async ({ parent, fetch, locals, url }) => {
	const { session } = await parent();

	const now = new Date();
	const office = resolveEmployeeOffice(session);
	const canCreateShipment = Boolean(office.id);
	const span = parseSpan(url.searchParams.get("span"));

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const [overview, shipments] = await Promise.all([
			getEmployeeOverview(client, span),
			listShipments(client),
		]);

		return {
			canCreateShipment,
			today: now.toISOString(),
			lastRefresh: now.toISOString(),
			greeting: getGreeting(session?.name ?? "there"),
			kpis: [
				{
					label: f("empd.actives"),
					value: overview.active_shipments,
					change: f("analytics.delta.vs_previous", {
						delta: signedDelta(overview.active_vs_last_period),
						period: periodLabel(span),
					}),
					context: f("analytics.context.office", {
						office: office.name ?? f("common.none"),
					}),
					trend: overview.active_vs_last_period >= 0 ? ("up" as const) : ("neutral" as const),
					severity: overview.active_vs_last_period >= 0 ? ("good" as const) : ("warn" as const),
					sparkline: overview.active_timeseries.map((point) => point.value),
				},
				{
					label: f("empd.pendings"),
					value: overview.pending_shipments,
					change: f("analytics.delta.vs_previous", {
						delta: signedDelta(overview.pending_vs_last_period),
						period: periodLabel(span),
					}),
					context: f("analytics.context.period", { period: periodLabel(span) }),
					trend:
						overview.pending_vs_last_period <= 0 ? ("up" as const) : ("neutral" as const),
					severity:
						overview.pending_shipments <= 3 ? ("good" as const) : ("warn" as const),
					sparkline: overview.pending_timeseries.map((point) => point.value),
				},
				{
					label: f("empd.deliveries"),
					value: overview.deliveries_today,
					change: f("analytics.delta.vs_yesterday", {
						delta: signedDelta(overview.deliveries_vs_last_period),
					}),
					context: f("analytics.context.period", { period: periodLabel(span) }),
					trend:
						overview.deliveries_vs_last_period >= 0
							? ("up" as const)
							: ("neutral" as const),
					severity: "good" as const,
					sparkline: overview.deliveries_timeseries.map((point) => point.value),
				},
			],
			recentShipments: shipments.slice(0, 5).map((shipment) => ({
				id: shipment.id,
				destination: shipment.office,
				status: dashboardStatus(shipment.status),
				eta: formatDashboardDate(shipment.updatedAt),
				priority:
					shipment.status === "pending" || shipment.status === "in_transit"
						? ("high" as const)
						: ("normal" as const),
			})),
			recentSearches: shipments.slice(0, 3).map((shipment) => shipment.id),
			activity: buildActivity(shipments),
		};
	} catch (error) {
		if (error instanceof HubApiError) {
			console.error('employee.dashboard.load failed', {
				status: error.status,
				code: error.code,
				message: error.message,
				upstream: error.upstream,
			});
		} else {
			console.error('employee.dashboard.load failed', error);
		}

		return {
			canCreateShipment,
			today: now.toISOString(),
			lastRefresh: now.toISOString(),
			greeting: getGreeting(session?.name ?? "there"),
			kpis: [
				{
					label: f("empd.actives"),
					value: 0,
					change: f("analytics.delta.vs_previous", {
						delta: "+0",
						period: periodLabel(span),
					}),
					context: f("analytics.context.office", {
						office: office.name ?? f("common.none"),
					}),
					trend: "neutral" as const,
					severity: "warn" as const,
					sparkline: [0, 0, 0, 0, 0, 0, 0],
				},
				{
					label: f("empd.pendings"),
					value: 0,
					change: f("analytics.delta.vs_previous", {
						delta: "0",
						period: periodLabel(span),
					}),
					context: f("analytics.context.period", { period: periodLabel(span) }),
					trend: "neutral" as const,
					severity: "warn" as const,
					sparkline: [0, 0, 0, 0, 0, 0, 0],
				},
				{
					label: f("empd.deliveries"),
					value: 0,
					change: f("analytics.delta.vs_yesterday", { delta: "0" }),
					context: f("analytics.context.period", { period: periodLabel(span) }),
					trend: "neutral" as const,
					severity: "warn" as const,
					sparkline: [0, 0, 0, 0, 0, 0, 0],
				},
			],
			recentShipments: [],
			recentSearches: [],
			activity: [],
		};
	}
};
