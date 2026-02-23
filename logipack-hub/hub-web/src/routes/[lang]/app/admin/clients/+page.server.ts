import type { PageServerLoad } from "./$types";
import {
	filterMockClientsByQuery,
	listMockClients,
} from "$lib/server/mockClients";

type ClientListItem = {
	id: string;
	name: string;
	email?: string | null;
	phone?: string | null;
};

async function fetchAdminClients(q: string): Promise<ClientListItem[]> {
	// TODO(api): replace mock with hub-api GET /admin/clients
	return filterMockClientsByQuery(listMockClients(), q);
}

export const load: PageServerLoad = async ({ url }) => {
	const q = url.searchParams.get("q")?.trim() ?? "";

	try {
		return {
			clients: await fetchAdminClients(q),
			q,
			loadError: false,
		};
	} catch {
		return {
			clients: [],
			q,
			loadError: true,
		};
	}
};
