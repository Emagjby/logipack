import { deleteMockOffice, getMockOfficeById } from "$lib/server/mockOffices";
import { fail, redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";
import type { Actions } from "./$types";

type OfficeDetail = {
	id: string;
	name: string;
	city: string;
	address: string;
	updated_at: string;
};

type DetailResult =
	| { state: "ok"; office: OfficeDetail }
	| { state: "not_found" }
	| { state: "error"; message: string };

function fetchOfficeDetail(id: string): DetailResult {
	const office = getMockOfficeById(id);
	if (!office) {
		return { state: "not_found" };
	}

	return {
		state: "ok",
		office: {
			id: office.id,
			name: office.name,
			city: office.city,
			address: office.address,
			updated_at: office.updated_at,
		},
	};
}

export const load: PageServerLoad = async ({ params }) => {
	try {
		return { result: fetchOfficeDetail(params.id) };
	} catch (error) {
		return {
			result: {
				state: "error" as const,
				message:
					error instanceof Error
						? error.message
						: "Unable to load office detail right now.",
			},
		};
	}
};

export const actions: Actions = {
	delete: async ({ params }) => {
		try {
			const deleted = deleteMockOffice(params.id);
			if (!deleted) {
				return fail(404, {
					submitError: "admin.offices.detail.not_found",
				});
			}
		} catch {
			return fail(500, {
				submitError: "admin.offices.detail.delete_failed",
			});
		}

		throw redirect(303, `/${params.lang ?? "en"}/app/admin/offices`);
	},
};
