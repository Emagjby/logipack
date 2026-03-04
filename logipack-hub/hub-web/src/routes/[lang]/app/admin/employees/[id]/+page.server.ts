import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import {
	deleteEmployee,
	getEmployee,
} from "$lib/server/hubApi/services/employees";

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const detail = await getEmployee(client, params.id);

		return {
			result: { state: "ok" as const, ...detail },
		};
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return { result: { state: "not_found" as const } };
		}

		if (e instanceof HubApiError) {
			console.error("admin.employees.detail.load_failed", {
				employeeId: params.id,
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.employees.detail.load_failed", {
				employeeId: params.id,
				e,
			});
		}

		return {
			result: {
				state: "error" as const,
				message: "admin.employees.detail.load_failed",
			},
		};
	}
};

export const actions: Actions = {
	delete: async ({ params, fetch, locals }) => {
		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			await deleteEmployee(client, params.id);

			throw redirect(303, `/${params.lang ?? "en"}/app/admin/employees`);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						submitError: "admin.employees.detail.not_found",
					});
				}

				return fail(500, {
					submitError: "admin.employees.detail.delete_failed",
				});
			}

			return fail(500, {
				submitError: "admin.employees.detail.delete_failed",
			});
		}
	},
};
