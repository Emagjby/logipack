import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	getAdminOverview,
	getReport,
	type AnalyticsSpan,
	type AuditEvent,
	HubApiError,
	listAuditEvents,
} from "$lib/server/hubApi";
import { normalizeShipmentStatus, statusLabelKey } from "$lib/domain/shipmentStatus";
import { compactId } from "$lib/utils/idDisplay";
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
	return raw === "7d" || raw === "90d" ? raw : "30d";
}

function periodLabel(span: AnalyticsSpan): string {
	return f(`analytics.period.${span}`);
}

function signedDelta(value: number): string {
	return value > 0 ? `+${value}` : `${value}`;
}

function trendOf(value: number): "up" | "neutral" {
	return value >= 0 ? "up" : "neutral";
}

function severityOf(value: number): "good" | "warn" {
	return value >= 0 ? "good" : "warn";
}

function statusAppearance(status: string) {
	switch (normalizeShipmentStatus(status)) {
		case "new":
			return { strokeClass: "text-sky-400", dotClass: "bg-sky-400" };
		case "accepted":
			return { strokeClass: "text-cyan-400", dotClass: "bg-cyan-400" };
		case "pending":
			return { strokeClass: "text-amber-400", dotClass: "bg-amber-400" };
		case "in_transit":
			return { strokeClass: "text-indigo-400", dotClass: "bg-indigo-400" };
		case "delivered":
			return { strokeClass: "text-accent", dotClass: "bg-accent" };
		case "cancelled":
		default:
			return { strokeClass: "text-rose-400", dotClass: "bg-rose-400" };
	}
}

function shortTime(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return "—";
	return new Intl.DateTimeFormat("en-GB", {
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
		timeZone: "UTC",
	}).format(date);
}

function formatAuditEntity(event: AuditEvent): string {
	if (event.entity_label && !event.entity_label.toLowerCase().startsWith(`${event.entity_type ?? ""} `)) {
		return event.entity_label;
	}
	return event.entity_id ?? f("common.none");
}

function formatCompactAuditEntity(event: AuditEvent): string {
	if (event.entity_label && !event.entity_label.toLowerCase().startsWith(`${event.entity_type ?? ""} `)) {
		return event.entity_label;
	}
	return event.entity_id ? compactId(event.entity_id) : f("common.none");
}

function formatAuditOffice(event: AuditEvent): string {
	return event.office_label ?? event.office_id ?? f("common.none");
}

function formatCompactAuditOffice(event: AuditEvent): string {
	return event.office_label ?? (event.office_id ? compactId(event.office_id) : f("common.none"));
}

function formatAuditStatus(event: AuditEvent, key: "from_status" | "to_status"): string {
	const value = event.metadata?.[key];
	if (typeof value !== "string" || value.trim() === "") {
		return f("shipment_status.unknown");
	}
	return f(statusLabelKey(normalizeShipmentStatus(value)));
}

function dotClassForEvent(event: AuditEvent): string {
	switch (event.entity_type) {
		case "shipment":
			return event.action_key === 'shipment.status_updated'
				? "bg-amber-400"
				: "bg-accent";
		case "client":
			return "bg-sky-400";
		case "office":
			return "bg-amber-400";
		case "employee":
			return "bg-violet-400";
		default:
			return "bg-surface-500";
	}
}

function eventHref(lang: string, event: AuditEvent): string {
	if (event.target_route) {
		return `/${lang}${event.target_route}`;
	}
	return `/${lang}/app/admin/audit`;
}

function mapRecentEvents(lang: string, events: AuditEvent[]) {
	return events.slice(0, 8).map((event) => {
		const fullValues = {
			entity: formatAuditEntity(event),
			office: formatAuditOffice(event),
			from_status: formatAuditStatus(event, "from_status"),
			to_status: formatAuditStatus(event, "to_status"),
		};

		return {
			id: event.id,
			eventKey: `admin.audit.action.${event.action_key}`,
			eventValues: {
				entity: formatCompactAuditEntity(event),
				office: formatCompactAuditOffice(event),
				from_status: fullValues.from_status,
				to_status: fullValues.to_status,
			},
			eventTitle: f(`admin.audit.action.${event.action_key}`, fullValues),
			actor:
				event.actor_display_name ??
				(event.actor_user_id ? compactId(event.actor_user_id) : f("admin.dashboard.actor.system")),
			actorTitle:
				event.actor_display_name ??
				event.actor_user_id ??
				f("admin.dashboard.actor.system"),
			time: shortTime(event.occurred_at),
			dotClass: dotClassForEvent(event),
			href: eventHref(lang, event),
		};
	});
}

export const load: PageServerLoad = async ({ fetch, locals, url, params }) => {
	const span = parseSpan(url.searchParams.get("span"));
	const now = new Date();

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const [overview, statusReport, audit] = await Promise.all([
			getAdminOverview(client, span),
			getReport(client, "shipments-by-status"),
			listAuditEvents(client, { limit: 8 }),
		]);

		const shipmentStatus = statusReport.rows.map((row) => {
			const status = typeof row[0] === "string" ? row[0] : "unknown";
			const count = typeof row[1] === "number" ? row[1] : 0;
			const appearance = statusAppearance(status);
			return {
				label: f(statusLabelKey(normalizeShipmentStatus(status))),
				value: count,
				strokeClass: appearance.strokeClass,
				dotClass: appearance.dotClass,
				href: `/${params.lang}/app/admin/shipments?status=${status.toLowerCase()}`,
			};
		});

		return {
			lastRefresh: now.toISOString(),
			kpis: [
				{
					label: f("admin.dashboard.kpi.total_shipments"),
					value: overview.total_shipments,
					change: f("analytics.delta.vs_previous", {
						delta: signedDelta(overview.shipments_vs_last_period),
						period: periodLabel(span),
					}),
					context: f("admin.dashboard.kpi.total_shipments_context", {
						count: overview.total_offices,
					}),
					trend: trendOf(overview.shipments_vs_last_period),
					severity: severityOf(overview.shipments_vs_last_period),
					sparkline: overview.shipments_timeseries.map((point) => point.value),
					href: `/${params.lang}/app/admin/shipments`,
				},
				{
					label: f("admin.dashboard.kpi.total_clients"),
					value: overview.total_clients,
					change: f("analytics.delta.vs_previous", {
						delta: signedDelta(overview.clients_vs_last_period),
						period: periodLabel(span),
					}),
					context: f("analytics.context.active_clients", {
						count: overview.total_clients,
					}),
					trend: trendOf(overview.clients_vs_last_period),
					severity: severityOf(overview.clients_vs_last_period),
					sparkline: overview.clients_timeseries.map((point) => point.value),
					href: `/${params.lang}/app/admin/clients`,
				},
				{
					label: f("admin.dashboard.kpi.total_offices"),
					value: overview.total_offices,
					change: f("analytics.delta.vs_previous", {
						delta: signedDelta(overview.offices_vs_last_period),
						period: periodLabel(span),
					}),
					context: f("analytics.context.active_offices", {
						count: overview.total_offices,
					}),
					trend: trendOf(overview.offices_vs_last_period),
					severity: severityOf(overview.offices_vs_last_period),
					sparkline: overview.offices_timeseries.map((point) => point.value),
					href: `/${params.lang}/app/admin/offices`,
				},
				{
					label: f("admin.dashboard.kpi.total_employees"),
					value: overview.total_employees,
					change: f("analytics.context.assigned_split", {
						assigned: overview.assigned_employees,
						unassigned: overview.unassigned_employees,
					}),
					context: f("analytics.context.employee_coverage", {
						count: overview.total_offices,
					}),
					trend: "neutral" as const,
					severity:
						overview.unassigned_employees === 0 ? ("good" as const) : ("warn" as const),
					sparkline: overview.employees_timeseries.map((point) => point.value),
					href: `/${params.lang}/app/admin/employees`,
				},
			],
			shipmentStatus,
			recentEvents: mapRecentEvents(params.lang, audit.events),
		};
	} catch (error) {
		if (error instanceof HubApiError) {
			console.error('admin.dashboard.load failed', {
				status: error.status,
				code: error.code,
				message: error.message,
				upstream: error.upstream,
			});
		} else {
			console.error('admin.dashboard.load failed', error);
		}

		return {
			lastRefresh: now.toISOString(),
			kpis: [],
			shipmentStatus: [],
			recentEvents: [],
		};
	}
};
