import { getMockClientById, updateMockClient } from "$lib/server/mockClients";
import {
	hasClientFormErrors,
	parseClientFormData,
	validateClientForm,
} from "$lib/server/clientForm";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

const EMPTY_VALUES = {
	name: "",
	email: null,
	phone: null,
};

export const load: PageServerLoad = async ({ params }) => {
	// TODO(api): replace mock with hub-api GET /admin/clients/:id
	const client = getMockClientById(params.id);
	if (!client) {
		return {
			clientId: params.id,
			notFound: true as const,
			initialValues: EMPTY_VALUES,
		};
	}

	return {
		clientId: client.id,
		notFound: false as const,
		initialValues: {
			name: client.name,
			email: client.email,
			phone: client.phone,
		},
	};
};

export const actions: Actions = {
	default: async ({ request, params }) => {
		const values = parseClientFormData(await request.formData());
		const fieldErrors = validateClientForm(values);

		if (hasClientFormErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		try {
			// TODO(api): replace mock update with hub-api PUT/PATCH /admin/clients/:id
			const updatedClient = updateMockClient(params.id, values);
			if (!updatedClient) {
				return fail(404, {
					fieldErrors: {},
					submitError: "admin.clients.detail.not_found",
					values,
				});
			}
		} catch {
			return fail(500, {
				fieldErrors: {},
				submitError: "admin.clients.edit.submit_failed",
				values,
			});
		}

		redirect(303, `/${params.lang ?? "en"}/app/admin/clients/${params.id}`);
	},
};
