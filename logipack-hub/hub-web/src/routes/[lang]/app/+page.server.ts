import type { PageServerLoad } from "./$types";
import { redirect } from "@sveltejs/kit";

export const load: PageServerLoad = async ({ locals, params }) => {
	const lang = params.lang;

	const role = locals.session?.role ?? "";

	switch (role) {
		case "admin":
			throw redirect(302, `/${lang}/app/admin`);
		case "employee":
			throw redirect(302, `/${lang}/app/employee`);
		default:
			throw redirect(302, `/${lang}/app/no-access`);
	}
};
