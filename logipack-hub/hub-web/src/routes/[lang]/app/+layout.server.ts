import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = async ({ locals, params, url }) => {
	const lang = params.lang;

	const session = locals.session;
	if (!session) {
		throw redirect(303, `/${lang}/login`);
	}

	const role = session.role ?? "";
	const isNoAccess = url.pathname === `/${lang}/app/no-access`;

	if (isNoAccess) {
		return { session, pathname: url.pathname };
	}

	if (role !== "admin" && role !== "employee") {
		throw redirect(303, `/${lang}/app/no-access`);
	}

	return {
		session,
		pathname: url.pathname,
	};
};
