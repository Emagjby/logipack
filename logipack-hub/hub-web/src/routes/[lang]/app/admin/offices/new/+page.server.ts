import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	createOffice,
	HubApiError,
} from "$lib/server/hubApi";
import {
	hasOfficeFormErrors,
	parseOfficeFormData,
	validateOfficeForm,
} from "$lib/server/officeForm";
import type { Actions } from "@sveltejs/kit";
import { fail, isRedirect, redirect } from "@sveltejs/kit";

export const actions: Actions = {
	default: async ({ request, params, fetch, locals }) => {
		const values = parseOfficeFormData(await request.formData());
		const fieldErrors = validateOfficeForm(values);

		if (hasOfficeFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			const created = await createOffice(client, values);
			const officeId = created.id;

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/offices/${officeId}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 400) {
					return fail(400, {
						fieldErrors: {},
						submitError: "admin.offices.new.submit_failed",
						values,
					});
				}

				return fail(500, {
					fieldErrors: {},
					submitError: "admin.offices.new.submit_failed",
					values,
				});
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.offices.new.submit_failed",
				values,
			});
		}
	},
};
