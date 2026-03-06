import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { listShipments } from "$lib/server/hubApi/services/shipments";
import type { PageServerLoad } from "./$types";
import { error } from "@sveltejs/kit";
import type { ShipmentRow } from "$lib/domain/shipmentStatus";

// ── Result discriminant returned to the page ────────────────────────
type ShipmentsResult =
	| { state: "ok"; shipments: ShipmentRow[] }
	| { state: "empty"; shipments: [] }
	| { state: "error"; shipments: []; message: string };

export const load: PageServerLoad = async ({ parent, fetch, locals }) => {
	const { session } = await parent();
	const activeOffice = "Sofia HQ";

	// Role guard: admin should not access employee pages
	if (session?.role === "admin") {
		throw error(403, "error.details.employee_only");
	}

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const shipments = await listShipments(client);

		const result: ShipmentsResult =
			shipments.length > 0
				? { state: "ok", shipments }
				: { state: "empty", shipments: [] };

		return { result, activeOffice, offices: [] };
	} catch (e) {
		if (e instanceof HubApiError) {
			console.error("employee.shipments.list failed", {
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("employee.shipments.list failed", e);
		}

		return {
			activeOffice,
			offices: [],
			result: {
				state: "error" as const,
				shipments: [] as [],
				message:
					e instanceof Error ? e.message : "shipments.error.generic",
			},
		};
	}
};
