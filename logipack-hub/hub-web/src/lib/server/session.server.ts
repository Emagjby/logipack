export type LpSession = {
	access_token: string;
	expires_at: number;
	role: string;
	name: string;
	email: string;

	refresh_token: string;
	id_token?: string;

	[key: string]: unknown;
};

function isObject(v: unknown): v is Record<string, unknown> {
	return !!v && typeof v === "object";
}

function fail(field: string, expected: string, received: unknown): null {
	console.error(`[parseSession] invalid ${field} (expected ${expected})`, {
		received,
	});
	return null;
}

export function parseSession(payload: unknown): LpSession | null {
	if (!isObject(payload)) {
		console.error("[parseSession] invalid payload (expected object)", {
			received: payload,
		});
		return null;
	}

	const access_token = payload.access_token;
	// TODO
	// const refresh_token = payload.refresh_token;
	const expires_at = payload.expires_at;
	const role = payload.role;
	const name = payload.name;
	const email = payload.email;
	const id_token = payload.id_token;

	if (typeof access_token !== "string" || !access_token)
		return fail("access_token", "non-empty string", access_token);

	// TODO
	// if (typeof refresh_token !== "string" || !refresh_token)
	// 	return fail("refresh_token", "non-empty string", refresh_token);

	if (typeof expires_at !== "number" || !Number.isFinite(expires_at))
		return fail("expires_at", "finite number", expires_at);

	if (typeof role !== "string") return fail("role", "string", role);

	if (typeof name !== "string") return fail("name", "string", name);

	if (typeof email !== "string") return fail("email", "string", email);

	if (id_token !== undefined && typeof id_token !== "string")
		return fail("id_token", "string | undefined", id_token);

	return {
		access_token,
		expires_at,
		role,
		name,
		email,
		// TODO: FIX THIS POST MVP
		refresh_token: "",
		id_token,
	};
}
