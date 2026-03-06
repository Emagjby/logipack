import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { listShipments } from "$lib/server/hubApi/services/shipments";
import { listOffices } from "$lib/server/hubApi/services/offices";
import type { PageServerLoad } from "./$types";
import type { ShipmentRow } from "$lib/domain/shipmentStatus";

type AdminShipmentsResult =
	| { state: "ok"; shipments: ShipmentRow[] }
	| { state: "empty"; shipments: [] }
	| { state: "error"; shipments: []; message?: string };

export const load: PageServerLoad = async ({ fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const [shipments, offices] = await Promise.all([
			listShipments(client),
			listOffices(client),
		]);

		const result: AdminShipmentsResult =
			shipments.length > 0
				? { state: "ok", shipments }
				: { state: "empty", shipments: [] };

		return { result, offices };
	} catch (e) {
		if (e instanceof HubApiError) {
			console.error("admin.shipments.list failed", {
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.shipments.list failed", e);
		}

		return {
			offices: [],
			result: {
				state: "error" as const,
				shipments: [] as [],
				message: "shipments.error.generic",
			},
		};
	}
};
