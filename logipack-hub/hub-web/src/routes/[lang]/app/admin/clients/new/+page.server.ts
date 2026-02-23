import { createMockClient } from "$lib/server/mockClients";
import {
	hasClientFormErrors,
	parseClientFormData,
	validateClientForm,
} from "$lib/server/clientForm";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";

export const actions: Actions = {
	default: async ({ request, params }) => {
		const values = parseClientFormData(await request.formData());
		const fieldErrors = validateClientForm(values);

		if (hasClientFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		let clientId: string;
		try {
			// TODO(api): replace mock create with hub-api POST /admin/clients
			const { id } = createMockClient(values);
			clientId = id;
		} catch {
			return fail(500, {
				fieldErrors: {},
				submitError: "admin.clients.new.submit_failed",
				values,
			});
		}

		throw redirect(303, `/${params.lang ?? "en"}/app/admin/clients/${clientId}`);
	},
};
