import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { listEmployees } from "$lib/server/hubApi/services/employees";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const employees = await listEmployees(client);

		return {
			employees,
			loadError: false,
		};
	} catch (e) {
		if (e instanceof HubApiError) {
			console.error("admin.employees.list failed", {
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.employees.list failed", e);
		}

		return {
			employees: [],
			loadError: true,
		};
	}
};
