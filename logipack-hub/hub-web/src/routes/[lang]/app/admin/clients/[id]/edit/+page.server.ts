import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { getClient, updateClient } from "$lib/server/hubApi/services/clients";
import { fail, isRedirect, redirect, type Actions } from "@sveltejs/kit";
import type { PageServerLoad } from "../$types";
import {
	hasClientFormErrors,
	parseClientFormData,
	validateClientForm,
} from "$lib/server/clientForm";

const EMPTY_VALUE = {
	name: "",
	email: null,
	phone: null,
};

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});
		const detail = await getClient(client, params.id);

		return {
			clientId: detail.id,
			notFound: false as const,
			initialValues: {
				name: detail.name,
				email: detail.email ?? null,
				phone: detail.phone ?? null,
			},
		};
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return {
				clientId: params.id,
				notFound: true as const,
				initialValues: EMPTY_VALUE,
			};
		}

		console.error("admin.clients.edit.load_failed", { clientId: params.id, e });
		return {
			clientId: params.id,
			notFound: false as const,
			initialValues: EMPTY_VALUE,
		};
	}
};

export const actions: Actions = {
	default: async ({ request, params, fetch, locals }) => {
		const values = parseClientFormData(await request.formData());
		const fieldErrors = validateClientForm(values);

		if (hasClientFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			await updateClient(client, params.id!, {
				name: values.name,
				email: values.email,
				phone: values.phone,
			});

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/clients/${params.id}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						fieldErrors: {},
						submitError: "admin.clients.detail.not_found",
						values,
					});
				}

				if (e.status === 400) {
					return fail(400, {
						fieldErrors: {},
						submitError: "admin.clients.edit.submit_failed",
						values,
					});
				}
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.clients.edit.submit_failed",
				values,
			});
		}
	},
};
