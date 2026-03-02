import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { listClients } from "$lib/server/hubApi/services/clients";
import type { PageServerLoad } from "./$types";

function filterByQuery<
	T extends {
		id?: string;
		name: string;
		email?: string | null;
		phone?: string | null;
	},
>(items: T[], q: string): T[] {
	const needle = q.toLowerCase();
	if (!needle) {
		return items;
	}

	return items.filter((item) => {
		`${item.id ?? ""}${item.name}${item.email ?? ""}${item.phone ?? ""}`
			.toLowerCase()
			.includes(needle);
	});
}

export const load: PageServerLoad = async ({ url, fetch, locals }) => {
	const q = url.searchParams.get("q")?.trim() ?? "";

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const clients = await listClients(client);

		return {
			clients: filterByQuery(clients, q),
			q,
			loadError: false,
		};
	} catch (e) {
		if (e instanceof HubApiError) {
			console.error("admin.clients.list failed", {
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.clients.list failed", e);
		}

		return {
			clients: [],
			q,
			loadError: true,
		};
	}
};
