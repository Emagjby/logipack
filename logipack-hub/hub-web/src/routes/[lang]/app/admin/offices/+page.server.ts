import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	HubApiError,
	listOffices,
} from "$lib/server/hubApi";
import type { PageServerLoad } from "./$types";

function filterByQuery<
	T extends { name: string; city: string; address: string },
>(offices: T[], query: string): T[] {
	const needle = query.trim().toLowerCase();
	if (!needle) return offices;

	return offices.filter((o) =>
		`${o.name} ${o.city} ${o.address}`.toLowerCase().includes(needle),
	);
}

export const load: PageServerLoad = async ({ url, fetch, locals }) => {
	const query = url.searchParams.get("q") || "";

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const offices = await listOffices(client);
		return {
			offices: filterByQuery(offices, query),
			query,
			loadError: false,
		};
	} catch (e) {
		if (e instanceof HubApiError) {
			console.error("admin.offices.list failed", {
				status: e.status,
				message: e.message,
				code: e.code,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.offices.list failed", e);
		}

		return {
			offices: [],
			query,
			loadError: true,
		};
	}
};
