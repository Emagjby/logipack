import { HUB_API_BASE } from "$env/static/private";
import {
	hasClientFormErrors,
	parseClientFormData,
	validateClientForm,
} from "$lib/server/clientForm";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { createClient } from "$lib/server/hubApi/services/clients";
import { fail, isRedirect, redirect, type Actions } from "@sveltejs/kit";

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

			const created = await createClient(client, {
				name: values.name,
				email: values.email,
				phone: values.phone,
			});

			throw redirect(
				303,
				`${params.lang ?? "en"}/app/admin/clients/${created.id}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError && e.status === 400) {
				return fail(400, {
					fieldErrors: {},
					submitError: "admin.clients.new.submit_failed",
					values,
				});
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.clients.new.submit_failed",
				values,
			});
		}
	},
};
