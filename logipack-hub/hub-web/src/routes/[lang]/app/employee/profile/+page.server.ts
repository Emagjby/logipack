import type { PageServerLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { decodeJwt } from "jose";
import { HUB_API_BASE } from "$env/static/private";
import { createHubApiClient } from "$lib/server/hubApi";
import { getMeContext } from "$lib/server/hubApi/services/identity";
import { resolveEmployeeOffice } from "$lib/server/employeeOffice";

type OfficeSummary = {
	id: string;
	name: string | null;
};

type JwtClaims = Record<string, unknown>;

const logoutRouteModules = import.meta.glob("/src/routes/logout/+server.ts");
const hasLogoutRoute = Object.keys(logoutRouteModules).length > 0;

function isObject(value: unknown): value is Record<string, unknown> {
	return !!value && typeof value === "object" && !Array.isArray(value);
}

function toNonEmptyString(value: unknown): string | null {
	if (typeof value !== "string") return null;
	const trimmed = value.trim();
	return trimmed.length > 0 ? trimmed : null;
}

function toStringArray(value: unknown): string[] {
	if (!Array.isArray(value)) return [];
	return value
		.map((entry) => toNonEmptyString(entry))
		.filter((entry): entry is string => !!entry);
}

function getFirstString(
	source: Record<string, unknown>,
	keys: string[],
): string | null {
	for (const key of keys) {
		const value = toNonEmptyString(source[key]);
		if (value) return value;
	}
	return null;
}

function uniqueStrings(values: string[]): string[] {
	return [...new Set(values)];
}

function getJwtClaims(rawSession: Record<string, unknown>): JwtClaims {
	const idToken = getFirstString(rawSession, ["id_token", "idToken"]);
	if (!idToken) return {};
	try {
		return decodeJwt(idToken) as JwtClaims;
	} catch {
		return {};
	}
}

function normalizeOffices(rawSession: Record<string, unknown>): OfficeSummary[] {
	const officeCandidates = [
		rawSession.offices,
		rawSession.office_details,
		rawSession.officeDetails,
	];

	for (const candidate of officeCandidates) {
		if (!Array.isArray(candidate)) continue;

		const items: OfficeSummary[] = [];
		for (const entry of candidate) {
			if (typeof entry === "string") {
				items.push({ id: entry, name: null });
				continue;
			}

			if (!isObject(entry)) continue;

			const id = getFirstString(entry, ["id", "office_id", "officeId"]);
			if (!id) continue;
			const name = getFirstString(entry, ["name", "office_name", "officeName"]);
			items.push({ id, name });
		}

		if (items.length > 0) return items;
	}

	return [];
}

export const load: PageServerLoad = async ({ parent, fetch, locals }) => {
	const { session, pathname } = await parent();

	if (session?.role === "admin") {
		throw error(403, "error.details.employee_only");
	}

	const rawSession: Record<string, unknown> = isObject(session) ? session : {};
	const claims = getJwtClaims(rawSession);

	const email =
		getFirstString(rawSession, ["email"]) ??
		(getFirstString(claims, ["email"]) as string | null);

	const userId =
		getFirstString(rawSession, ["user_id", "userId", "sub", "id"]) ??
		getFirstString(claims, ["sub"]);

	const roles = uniqueStrings(
		[
			...toStringArray(rawSession["roles"]),
			...toStringArray(claims["roles"]),
			...toStringArray(claims["https://logipack/roles"]),
			...(toNonEmptyString(rawSession["role"])
				? [rawSession["role"] as string]
				: []),
		],
	);

    let employeeId =
		getFirstString(rawSession, ["employee_id", "employeeId"]) ??
		getFirstString(claims, ["employee_id", "employeeId"]);

	let offices = normalizeOffices(rawSession);
	let officeIds = uniqueStrings([
		...offices.map((office) => office.id),
		...toStringArray(rawSession["office_ids"]),
		...toStringArray(rawSession["officeIds"]),
		...toStringArray(rawSession["office_ids_list"]),
		...toStringArray(rawSession["officeIdsList"]),
	]);

	const singletonOfficeId =
		getFirstString(rawSession, ["office_id", "officeId"]) ??
		getFirstString(claims, ["office_id", "officeId"]);
	if (singletonOfficeId) {
		officeIds = uniqueStrings([...officeIds, singletonOfficeId]);
	}

	if (offices.length === 0 && officeIds.length > 0) {
		offices = officeIds.map((id) => ({ id, name: null }));
	}

	const resolvedOffice = resolveEmployeeOffice(rawSession);
	let currentOfficeId = resolvedOffice.id;
	let currentOfficeName = resolvedOffice.name;

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});
		const me = await getMeContext(client, 5_000);
		if (me.employee_id) {
			employeeId = me.employee_id;
		}
		if (me.current_office_id) {
			currentOfficeId = me.current_office_id;
		}
		if (me.current_office_name) {
			currentOfficeName = me.current_office_name;
		}
		if (me.office_ids.length > 0) {
			officeIds = uniqueStrings([...officeIds, ...me.office_ids]);
		}
	} catch (error) {
		console.error("employee.profile.me_failed", error);
	}

	if (!currentOfficeId && officeIds.length > 0) {
		currentOfficeId = officeIds[0] ?? null;
	}

	if (currentOfficeId && !currentOfficeName) {
		currentOfficeName =
			offices.find((office) => office.id === currentOfficeId)?.name ?? null;
	}

	return {
		pathname,
		hasLogoutRoute,
		profile: {
			email,
			userId,
			roles,
			employeeId,
			officeIds,
			offices,
			currentOffice: currentOfficeId
				? {
						id: currentOfficeId,
						name: currentOfficeName,
					}
				: null,
		},
	};
};
