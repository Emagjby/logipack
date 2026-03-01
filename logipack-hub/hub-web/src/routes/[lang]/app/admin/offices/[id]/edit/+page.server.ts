import { HUB_API_BASE } from "$env/static/private";
import {
	createHubApiClient,
	getOffice,
	HubApiError,
	updateOffice,
} from "$lib/server/hubApi";
import {
	hasOfficeFormErrors,
	parseOfficeFormData,
	validateOfficeForm,
} from "$lib/server/officeForm";
import { isRedirect, redirect, type Actions } from "@sveltejs/kit";
import type { PageServerLoad } from "../$types";
import { fail } from "@sveltejs/kit";

const EMPTY_VALUES = {
	name: "",
	address: "",
	city: "",
};

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const office = await getOffice(client, params.id);

		return {
			officeId: office.id,
			notFound: false as const,
			initialValues: {
				name: office.name,
				address: office.address,
				city: office.city,
			},
		};
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return {
				officeId: params.id,
				notFound: true as const,
				initialValues: EMPTY_VALUES,
			};
		}

		console.error("admin.offices.edit.load_failed", { officeId: params.id, e });

		return {
			officeId: params.id,
			notFound: false as const,
			initialValues: EMPTY_VALUES,
		};
	}
};

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

			await updateOffice(client, params.id!, values);

			throw redirect(303, `/${params.lang ?? "en"}/app/admin/offices`);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						fieldErrors: {},
						submitError: "admin.offices.edit.not_found",
						values,
					});
				}

				if (e.status === 400) {
					return fail(400, {
						fieldErrors: {},
						submitError: "admin.offices.edit.submit_failed",
						values,
					});
				}

				return fail(500, {
					fieldErrors: {},
					submitError: "admin.offices.edit.submit_failed",
					values,
				});
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.offices.edit.submit_failed",
				values,
			});
		}
	},
};
