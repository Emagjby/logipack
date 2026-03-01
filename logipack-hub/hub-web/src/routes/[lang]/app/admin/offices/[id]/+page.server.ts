import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";
import type { Actions } from "./$types";

import { HUB_API_BASE } from "$env/static/private";
import type { OfficeDetail } from "$lib/server/hubApi";
import {
	createHubApiClient,
	HubApiError,
	getOffice,
	deleteOffice,
} from "$lib/server/hubApi";

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const office = await getOffice(client, params.id);

		return {
			result: { state: "ok" as const, office },
		};
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return { result: { state: "not_found" as const } };
		}

		if (e instanceof HubApiError) {
			console.error("admin.offices.detail failed", {
				officeId: params.id,
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});

			return {
				result: {
					state: "error" as const,
					message: "Unable to load office detail right now.",
				},
			};
		}

		console.error("admin.offices.detail failed", { officeId: params.id, e });
		return {
			result: {
				state: "error" as const,
				message:
					e instanceof Error
						? e.message
						: "Unable to load office detail right now.",
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

			await deleteOffice(client, params.id);

			throw redirect(303, `/${params.lang ?? "en"}/app/admin/offices`);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						submitError: "admin.offices.detail.not_found",
					});
				}

				return fail(500, {
					submitError: "admin.offices.detail.delete_failed",
				});
			}

			return fail(500, {
				submitError: "admin.offices.detail.delete_failed",
			});
		}
	},
};
