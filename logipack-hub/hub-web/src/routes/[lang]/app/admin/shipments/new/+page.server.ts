import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { createShipment } from "$lib/server/hubApi/services/shipments";
import {
	hasShipmentCreateErrors,
	parseShipmentCreateFormData,
	validateShipmentCreateForm,
} from "$lib/server/shipmentCreateForm";
import { fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions } from "./$types";

export const actions: Actions = {
	default: async ({ request, params, fetch, locals }) => {
		const values = parseShipmentCreateFormData(await request.formData());
		const fieldErrors = validateShipmentCreateForm(values);

		if (hasShipmentCreateErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			const { id: shipmentId } = await createShipment(client, {
				client_id: values.client_id,
				current_office_id: values.current_office_id || null,
				notes: values.notes || null,
			});

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/admin/shipments/${shipmentId}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			console.error("admin.shipments.new.submit_failed", {
				client_id: values.client_id,
				current_office_id: values.current_office_id,
				e,
			});

			if (e instanceof HubApiError && e.status === 400) {
				return fail(400, {
					fieldErrors: {},
					submitError: "admin.shipments.new.submit_failed",
					values,
				});
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "admin.shipments.new.submit_failed",
				values,
			});
		}
	},
};
