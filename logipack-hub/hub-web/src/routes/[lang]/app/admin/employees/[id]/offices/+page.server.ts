import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import {
	assignEmployeeOffice,
	listEmployeeOffices,
	removeEmployeeOffice,
} from "$lib/server/hubApi/services/employeeOffices";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

type AssignOfficeValues = {
	office_id: string;
};

type AssignOfficeFieldErrors = {
	office_id?: string;
};

function parseAssignOfficeFormData(formData: FormData): AssignOfficeValues {
	return {
		office_id: String(formData.get("office_id") ?? "").trim(),
	};
}

function validateAssignOfficeForm(
	values: AssignOfficeValues,
): AssignOfficeFieldErrors {
	const fieldErrors: AssignOfficeFieldErrors = {};

	if (!values.office_id) {
		fieldErrors.office_id = "admin.employees.offices.form.office_required";
	}

	return fieldErrors;
}

function hasAssignOfficeFormErrors(
	fieldErrors: AssignOfficeFieldErrors,
): boolean {
	return Object.values(fieldErrors).some((value) => Boolean(value));
}

export const load: PageServerLoad = async ({ params, fetch, locals }) => {
	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});
		return {
			result: await listEmployeeOffices(client, params.id),
		};
	} catch (error) {
		if (error instanceof HubApiError && error.status === 404) {
			return { result: { state: "not_found" as const } };
		}

		return {
			result: {
				state: "error" as const,
				message: "admin.employees.offices.load_failed",
			},
		};
	}
};

export const actions: Actions = {
	assign: async ({ request, params, fetch, locals }) => {
		const values = parseAssignOfficeFormData(await request.formData());
		const fieldErrors = validateAssignOfficeForm(values);

		if (hasAssignOfficeFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, submitError: null, values });
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});
			const context = await listEmployeeOffices(client, params.id);
			const currentOfficeId =
				context.state === "ok" ? context.currentOfficeId : null;

			if (currentOfficeId && currentOfficeId !== values.office_id) {
				await removeEmployeeOffice(client, params.id, currentOfficeId);
			}

			if (!currentOfficeId || currentOfficeId !== values.office_id) {
				await assignEmployeeOffice(client, params.id, values.office_id);
			}
		} catch (error) {
			if (error instanceof HubApiError) {
				if (error.status === 404) {
					return fail(404, {
						fieldErrors: {},
						submitError: "admin.employees.detail.not_found",
						values,
					});
				}

				if (error.status === 409) {
					return fail(409, {
						fieldErrors: {
							office_id: "admin.employees.offices.form.office_already_assigned",
						} as AssignOfficeFieldErrors,
						submitError: null,
						values,
					});
				}

				if (error.status === 400) {
					return fail(400, {
						fieldErrors: {
							office_id: "admin.employees.offices.form.office_invalid",
						} as AssignOfficeFieldErrors,
						submitError: null,
						values,
					});
				}
			}

			return fail(500, {
				fieldErrors: {} as AssignOfficeFieldErrors,
				submitError: "admin.employees.offices.submit_failed",
				values,
			});
		}

		throw redirect(
			303,
			`/${params.lang ?? "en"}/app/admin/employees/${params.id}/offices`,
		);
	},
};
