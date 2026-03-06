import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient, HubApiError } from "$lib/server/hubApi";
import { createShipment } from "$lib/server/hubApi/services/shipments";
import { getMeContext } from "$lib/server/hubApi/services/identity";
import { resolveEmployeeOffice } from "$lib/server/employeeOffice";
import {
	hasShipmentCreateErrors,
	parseShipmentCreateFormData,
	validateShipmentCreateForm,
} from "$lib/server/shipmentCreateForm";
import { error, fail, isRedirect, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

async function resolveEmployeeOfficeWithFallback(args: {
	session: unknown;
	fetch: typeof globalThis.fetch;
	locals: App.Locals;
}): Promise<{ id: string | null; name: string | null }> {
	const fromSession = resolveEmployeeOffice(args.session);
	if (fromSession.id) return fromSession;

	try {
		const client = createHubApiClient({
			fetch: args.fetch,
			locals: args.locals,
			baseUrl: HUB_API_BASE,
		});
		const me = await getMeContext(client, 5_000);
		const fallbackId = me.current_office_id ?? me.office_ids[0] ?? null;
		return {
			id: fallbackId,
			name: fromSession.name,
		};
	} catch {
		return fromSession;
	}
}

export const load: PageServerLoad = async ({ parent, fetch, locals }) => {
	const { session } = await parent();

	if (session?.role === "admin") {
		throw error(403, "error.details.employee_only");
	}

	const office = await resolveEmployeeOfficeWithFallback({
		session,
		fetch,
		locals,
	});
	return {
		office: {
			assignedId: office.id ?? "",
			label: office.name ?? office.id ?? null,
			isAvailable: Boolean(office.id),
			isLoading: false,
		},
	};
};

export const actions: Actions = {
	default: async ({ locals, params, request, fetch }) => {
		const session = (locals.session ?? null) as { role?: string } | null;
		if (session?.role === "admin") {
			throw error(403, "error.details.employee_only");
		}

		const office = await resolveEmployeeOfficeWithFallback({
			session,
			fetch,
			locals,
		});

		const values = parseShipmentCreateFormData(await request.formData());
		values.current_office_id = office.id ?? "";

		const fieldErrors = validateShipmentCreateForm(values);
		if (hasShipmentCreateErrors(fieldErrors)) {
			return fail(400, { fieldErrors, values });
		}
		if (!office.id) {
			return fail(422, {
				fieldErrors: {},
				submitError: "employee.shipments.new.office_required",
				values,
			});
		}

		try {
			const client = createHubApiClient({
				fetch,
				locals,
				baseUrl: HUB_API_BASE,
			});

			const { id: shipmentId } = await createShipment(client, {
				client_id: values.client_id,
				current_office_id: office.id,
				notes: values.notes || null,
			});

			throw redirect(
				303,
				`/${params.lang ?? "en"}/app/employee/shipments/${shipmentId}`,
			);
		} catch (e) {
			if (isRedirect(e)) {
				throw e;
			}

			console.error("employee.shipments.new.submit_failed", {
				client_id: values.client_id,
				office_id: office.id,
				e,
			});

			if (e instanceof HubApiError && e.status === 400) {
				return fail(400, {
					fieldErrors: {},
					submitError: "employee.shipments.new.submit_failed",
					values,
				});
			}

			return fail(500, {
				fieldErrors: {},
				submitError: "employee.shipments.new.submit_failed",
				values,
			});
		}
	},
};
