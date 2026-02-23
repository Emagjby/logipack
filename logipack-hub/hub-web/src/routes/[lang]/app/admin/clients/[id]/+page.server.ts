import { deleteMockClient, getMockClientById } from "$lib/server/mockClients";
import { fail, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

type ClientDetail = {
	id: string;
	name: string;
	email: string | null;
	phone: string | null;
	updated_at: string;
};

type DetailResult =
	| { state: "ok"; client: ClientDetail }
	| { state: "not_found" }
	| { state: "error"; message: string };

function fetchClientDetail(id: string): DetailResult {
	// TODO(api): replace mock with hub-api GET /admin/clients/:id
	const client = getMockClientById(id);
	if (!client) {
		return { state: "not_found" };
	}

	return {
		state: "ok",
		client: {
			id: client.id,
			name: client.name,
			email: client.email,
			phone: client.phone,
			updated_at: client.updated_at,
		},
	};
}

export const load: PageServerLoad = async ({ params }) => {
	try {
		return { result: fetchClientDetail(params.id) };
	} catch (error) {
		console.error("admin.clients.detail.load_failed", {
			clientId: params.id,
			error,
		});
		return {
			result: {
				state: "error" as const,
				message: "admin.clients.detail.load_failed",
			},
		};
	}
};

export const actions: Actions = {
	delete: async ({ params }) => {
		try {
			const deleted = deleteMockClient(params.id);
			if (!deleted) {
				return fail(404, {
					submitError: "admin.clients.detail.not_found",
				});
			}
		} catch {
			return fail(500, {
				submitError: "admin.clients.detail.delete_failed",
			});
		}

		throw redirect(303, `/${params.lang ?? "en"}/app/admin/clients`);
	},
};
