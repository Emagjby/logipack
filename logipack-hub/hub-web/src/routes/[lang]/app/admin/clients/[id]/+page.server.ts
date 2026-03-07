import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { getClient } from "$lib/server/hubApi/services/clients";
import { fail, isRedirect, redirect, type Actions } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";

type DetailResult =
	| {
		state: "ok";
		client: Awaited<ReturnType<typeof getClient>>;
	}
	| {
		state: "not_found";
	}
	| {
		state: "error";
		message: string;
	};

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});
		const detail = await getClient(client, params.id);

		return { result: { state: "ok", client: detail } satisfies DetailResult };
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return { result: { state: "not_found" } satisfies DetailResult };
		}

		if (e instanceof HubApiError) {
			console.error("admin.clients.detail failed", {
				clientId: params.id,
				status: e.status,
				code: e.code,
				message: e.message,
				upstream: e.upstream,
			});
		} else {
			console.error("admin.clients.detail failed", {
				clientId: params.id,
				error: e,
			});
		}

		return {
			result: {
				state: "error",
				message: "admin.clients.detail.load_failed",
			} satisfies DetailResult,
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
			await client.delete(`/admin/clients/${params.id}`);

			throw redirect(303, `/${params.lang ?? "en"}/app/admin/clients`);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError && e.status === 404) {
				return fail(404, { submitError: "admin.clients.detail.not_found" });
			}

			return fail(500, { submitError: "admin.clients.detail.delete_failed" });
		}
	},
};
