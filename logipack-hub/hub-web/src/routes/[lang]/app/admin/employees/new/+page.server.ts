import {
	hasEmployeeFormErrors,
	parseEmployeeFormData,
	validateEmployeeForm,
} from "$lib/server/employeeForm";
import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { createEmployee } from "$lib/server/hubApi/services/employees";
import { HUB_API_BASE } from "$env/static/private";

const EMPTY_VALUES = {
	email: "",
};

export const load: PageServerLoad = async () => {
	return {
		initialValues: EMPTY_VALUES,
	};
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
			const created = await createEmployee(client, { email: values.email });
			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/employees/${created.id}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			if (e instanceof HubApiError) {
				if (e.status === 404) {
					return fail(404, {
						fieldErrors: { email: "employee.form.email_not_found" },
						values,
					});
				}

				if (e.status === 400) {
					return fail(400, {
						fieldErrors: {},
						submitError: "admin.employees.new.submit_failed",
						values,
					});
				}
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.employees.new.submit_failed",
				values,
			});
		}
	},
};
