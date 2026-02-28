/**
 * Extract bearer access token from SSR locals.
 *
 * Rules:
 * - Only reads locals.session.access_token
 * - Does not inspect cookies/headers/url
 * - Returns null unless it's a non-empty string
 */
export function getAccessTokenFromLocals(locals: App.Locals): string | null {
	const token = locals.session?.access_token;
	return validate_access_token(token);
}

function validate_access_token(token: unknown): string | null {
	if (typeof token === "string" && token.trim() !== "") {
		return token;
	}
	return null;
}
