export type LpSession = {
	access_token: string;
	expires_at: number;
	role: string;
	name: string;
	email: string;

	refresh_token?: string;
	id_token?: string;

	[key: string]: unknown;
};

function isObject(v: unknown): v is Record<string, unknown> {
	return !!v && typeof v === "object" && !Array.isArray(v);
}

// TODO: refactor later
function fail(field: string, expected: string, received: unknown): null {
	console.error(`[parseSession] invalid ${field} (expected ${expected})`, {
		received,
	});
	return null;
}

function nonEmptyStringOrUndefined(v: unknown): string | undefined {
	if (typeof v !== "string") return undefined;
	const s = v.trim();

	if (s.length === 0) return undefined;
	return s;
}

export function parseSession(payload: unknown): LpSession | null {
	// TODO: refactor later
	if (!isObject(payload)) {
		console.error("[parseSession] invalid payload (expected object)", {
			received: payload,
		});
		return null;
	}

	const access_token = payload.access_token;
	const expires_at = payload.expires_at;

	const role = payload.role;
	const name = payload.name;
	const email = payload.email;

	const id_token = nonEmptyStringOrUndefined(payload.id_token);
	const refresh_token = nonEmptyStringOrUndefined(payload.refresh_token);

	if (typeof access_token !== "string" || !access_token)
		return fail("access_token", "non-empty string", access_token);

	if (typeof expires_at !== "number" || !Number.isFinite(expires_at))
		return fail("expires_at", "finite number", expires_at);

	if (typeof role !== "string") return fail("role", "string", role);

	if (typeof name !== "string") return fail("name", "string", name);

	if (typeof email !== "string") return fail("email", "string", email);

	return {
		access_token,
		expires_at,
		role,
		name,
		email,
		...(id_token ? { id_token } : {}),
		...(refresh_token ? { refresh_token } : {}),
	};
}
