import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import {
	hasEmployeeFormErrors,
	parseEmployeeFormData,
	validateEmployeeForm,
} from "$lib/server/employeeForm";
import {
	getEmployee,
	updateEmployee,
} from "$lib/server/hubApi/services/employees";
import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

const EMPTY_VALUES = {
	email: "",
};

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});
		const detail = await getEmployee(client, params.id);

		return {
			employeeId: detail.id,
			notFound: false as const,
			initialValues: {
				email: detail.email,
			},
		};
	} catch (e) {
		if (e instanceof HubApiError && e.status === 404) {
			return {
				employeeId: params.id,
				notFound: true as const,
				initialValues: EMPTY_VALUES,
			};
		}

		console.error("admin.employees.edit.load_failed", {
			employeeId: params.id,
			e,
		});

		return {
			employeeId: params.id,
			notFound: false as const,
			initialValues: EMPTY_VALUES,
		};
	}
};

export const actions: Actions = {
	default: async ({ request, params, fetch, locals }) => {
		const values = parseEmployeeFormData(await request.formData());
		const fieldErrors = validateEmployeeForm(values);

		if (hasEmployeeFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			await updateEmployee(client, params.id!, { email: values.email });

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/employees/${params.id}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						fieldErrors: {},
						submitError: "admin.employees.detail.not_found",
						values,
					});
				}

				if (e.status === 400) {
					return fail(400, {
						fieldErrors: {},
						submitError: "admin.employees.edit.submit_failed",
						values,
					});
				}
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.employees.edit.submit_failed",
				values,
			});
		}
	},
};
