import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import {
	getShipment,
	getShipmentTimeline,
	changeShipmentStatus,
} from "$lib/server/hubApi/services/shipments";
import {
	buildStatusHistory,
	buildStrataPackages,
	deriveCurrentOfficeIdFromTimeline,
} from "$lib/server/hubApi/mappers/shipments";
import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";
import {
	normalizeShipmentStatus,
	type ShipmentStatus,
} from "$lib/domain/shipmentStatus";
import type { StrataPackage } from "$lib/domain/strataPackage";

type ShipmentCore = {
	id: string;
	client_id: string;
	current_status: ShipmentStatus | "unknown";
	current_office_id: string | null;
	created_at: string;
	updated_at: string;
};

type StatusHistoryRow = {
	id: string;
	from_status: ShipmentStatus | "unknown" | null;
	to_status: ShipmentStatus | "unknown";
	changed_at: string;
	actor_user_id: string | null;
	office_id: string | null;
	notes: string | null;
};

type DetailResult =
	| {
		state: "ok";
		shipment: ShipmentCore;
		statusHistory: StatusHistoryRow[];
		packages: StrataPackage[];
	}
	| { state: "not_found" }
	| { state: "error"; message: string };

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const timelineQuery = new URLSearchParams({ format: "PRETTY" });

		const [detail, timeline] = await Promise.all([
			getShipment(client, params.id),
			getShipmentTimeline(client, params.id, timelineQuery),
		]);

		const statusHistory = buildStatusHistory(detail.id, detail, timeline);
		const packages = buildStrataPackages(detail.id, timeline);
		const currentOfficeFromStrata = deriveCurrentOfficeIdFromTimeline(
			timeline,
			detail.current_office_id,
		);

		const result: DetailResult = {
			state: "ok",
			shipment: {
				id: detail.id,
				client_id: detail.client_id,
				current_status: detail.current_status,
				current_office_id: currentOfficeFromStrata,
				created_at: detail.created_at,
				updated_at: detail.updated_at,
			},
			statusHistory,
			packages,
		};

		return { result };
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return { result: { state: "not_found" as const } };
		}

		if (e instanceof HubApiError) {
			console.error("admin.shipments.detail failed", {
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.shipments.detail failed", e);
		}

		return {
			result: {
				state: "error" as const,
				message: "shipment.detail.load_failed",
			},
		};
	}
};

export const actions: Actions = {
	changeStatus: async ({ request, params, fetch, locals }) => {
		const formData = await request.formData();
		const toStatus =
			(formData.get("new_status") as string | null)?.trim() ?? "";
		const rawOfficeId =
			(formData.get("office_id") as string | null)?.trim() || null;
		const notes = (formData.get("notes") as string | null)?.trim() || null;

		if (!toStatus) {
			return fail(400, {
				changeStatusError: "shipment.update.invalid_status",
				values: { new_status: toStatus, office_id: rawOfficeId ?? "", notes },
			});
		}

		const normalizedStatus = normalizeShipmentStatus(toStatus);
		if (normalizedStatus === "unknown") {
			return fail(400, {
				changeStatusError: "shipment.update.invalid_status",
				values: { new_status: toStatus, office_id: rawOfficeId ?? "", notes },
			});
		}

		if (normalizedStatus === "in_transit" && !rawOfficeId) {
			return fail(400, {
				changeStatusError: "shipment.update.office_required",
				values: { new_status: toStatus, office_id: "", notes },
			});
		}

		const toOfficeId = normalizedStatus === "in_transit" ? rawOfficeId : null;

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			await changeShipmentStatus(client, params.id, {
				to_status: normalizedStatus,
				to_office_id: toOfficeId,
				notes,
			});

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/shipments/${params.id}`,
			);
		} catch (e) {
			if (isRedirect(e)) throw e;

			console.error("admin.shipments.changeStatus failed", {
				shipment_id: params.id,
				to_status: normalizedStatus,
				e,
			});

			return fail(e instanceof HubApiError ? Math.max(e.status, 400) : 500, {
				changeStatusError: "shipment.update.failed",
				values: { new_status: toStatus, office_id: rawOfficeId ?? "", notes },
			});
		}
	},
};
