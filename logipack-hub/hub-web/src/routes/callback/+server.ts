import type { RequestHandler } from "./$types";
import { redirect, error, isHttpError, isRedirect } from "@sveltejs/kit";
import {
	AUTH0_DOMAIN,
	AUTH0_CLIENT_ID,
	AUTH0_CLIENT_SECRET,
	AUTH0_CALLBACK_URL,
	SESSION_SECRET,
	HUB_API_BASE,
} from "$env/static/private";

import { createHubApiClient } from "$lib/server/hubApi/httpClient";
import { HubApiError } from "$lib/server/hubApi/errors";
import { ensureUser, getMeContext } from "$lib/server/hubApi/services/identity";

import { EncryptJWT } from "jose";

function safeRedirectPath(raw: string, fallback = "/app"): string {
	try {
		const decoded = decodeURIComponent(raw);
		if (decoded.startsWith("/") && !decoded.startsWith("//")) return decoded;
	} catch { }
	return fallback;
}

function withQuery(path: string, params: Record<string, string>) {
	const u = new URL(path, "http://local");
	for (const [k, v] of Object.entries(params)) u.searchParams.set(k, v);
	return u.pathname + u.search;
}

const enc = new TextEncoder();
const SUPPORTED_LANGS = new Set(["en", "bg"]);

const AUTH_ERROR_DETAILS = {
	managementTokenFailed: "error.details.auth_management_token_failed",
	userProfileReadFailed: "error.details.auth_user_profile_read_failed",
	userRoleLoadFailed: "error.details.auth_user_role_load_failed",
	missingCode: "error.details.auth_missing_code",
	tokenExchangeFailed: "error.details.auth_exchange_failed",
	provisionFailed: "error.details.auth_provision_failed",
	accountConflict: "error.details.auth_account_conflict",
	invalidProfile: "error.details.auth_invalid_profile",
	unknown: "error.details.auth_unknown",
} as const;

function authErrorRedirectPath(lang: string, status: number, detail: string) {
	return withQuery(`/${lang}/auth/error`, {
		status: String(status),
		detail,
	});
}

async function deriveKey(secret: string): Promise<Uint8Array> {
	const data = enc.encode(secret);
	const hashBuffer = await crypto.subtle.digest("SHA-256", data);
	return new Uint8Array(hashBuffer);
}

function decodeJwtPayload(jwt: string): Record<string, unknown> {
	const parts = jwt.split(".");
	if (parts.length !== 3) return {};
	try {
		const payload = parts[1]!;
		const json = Buffer.from(payload, "base64url").toString("utf-8");
		return JSON.parse(json);
	} catch {
		return {};
	}
}

async function getMgmtToken(signal: AbortSignal): Promise<string> {
	const res = await fetch(`https://${AUTH0_DOMAIN}/oauth/token`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		signal,
		body: JSON.stringify({
			grant_type: "client_credentials",
			client_id: AUTH0_CLIENT_ID,
			client_secret: AUTH0_CLIENT_SECRET,
			audience: `https://${AUTH0_DOMAIN}/api/v2/`,
		}),
	});

	if (!res.ok) {
		const body = await res.text();
		console.error("MGMT token failed:", res.status, body);
		throw error(502, AUTH_ERROR_DETAILS.managementTokenFailed);
	}

	const json = (await res.json()) as { access_token: string };
	return json.access_token;
}

async function getUserFromMgmt(
	userId: string,
	mgmtToken: string,
	signal: AbortSignal,
) {
	const res = await fetch(
		`https://${AUTH0_DOMAIN}/api/v2/users/${encodeURIComponent(userId)}`,
		{
			headers: { Authorization: `Bearer ${mgmtToken}` },
			signal,
		},
	);

	if (!res.ok) {
		const body = await res.text();
		console.error("MGMT user fetch failed:", res.status, body);
		throw error(502, AUTH_ERROR_DETAILS.userProfileReadFailed);
	}

	return (await res.json()) as {
		name?: string;
		nickname?: string;
		email?: string;
	};
}

export const GET: RequestHandler = async ({ url, cookies, locals }) => {
	const cookieLang = cookies.get("lang");
	const lang =
		locals.lang ??
		(cookieLang && SUPPORTED_LANGS.has(cookieLang) ? cookieLang : "en");

	const rawState = url.searchParams.get("state");
	const state = rawState === "/app" || !rawState ? `/${lang}/app` : rawState;

	try {
		const code = url.searchParams.get("code");
		if (!code) throw error(400, AUTH_ERROR_DETAILS.missingCode);

		const tokenRes = await fetch(`https://${AUTH0_DOMAIN}/oauth/token`, {
			method: "POST",
			headers: { "content-type": "application/json" },
			signal: AbortSignal.timeout(10000),
			body: JSON.stringify({
				grant_type: "authorization_code",
				client_id: AUTH0_CLIENT_ID,
				client_secret: AUTH0_CLIENT_SECRET,
				code,
				redirect_uri: AUTH0_CALLBACK_URL,
			}),
		});

		if (!tokenRes.ok) {
			const body = await tokenRes.text();
			console.error("Token exchange failed:", tokenRes.status, body);
			throw error(502, AUTH_ERROR_DETAILS.tokenExchangeFailed);
		}

		const tokens = (await tokenRes.json()) as {
			access_token: string;
			id_token?: string;
			refresh_token?: string;
			expires_in: number;
			token_type: string;
		};

		const hub = createHubApiClient({
			fetch,
			locals: { session: { access_token: tokens.access_token } } as any,
			baseUrl: HUB_API_BASE,
			timeoutMs: 10_000,
		});

		let name = "";
		let email = "";
		let sub = "";

		if (tokens.id_token) {
			const claims = decodeJwtPayload(tokens.id_token);
			sub = (claims.sub as string) ?? "";
			name = (claims.name as string) ?? (claims.nickname as string) ?? "";
			email = (claims.email as string) ?? "";
		}

		const looksBad =
			!name ||
			(!!name &&
				!!email &&
				name.trim().toLowerCase() === email.trim().toLowerCase());

		if (looksBad && sub) {
			const mgmtToken = await getMgmtToken(AbortSignal.timeout(10000));
			const u = await getUserFromMgmt(
				sub,
				mgmtToken,
				AbortSignal.timeout(10000),
			);
			name = u.name ?? u.nickname ?? name;
			email = u.email ?? email;
		}

		if (!name || !email) {
			const userinfoRes = await fetch(`https://${AUTH0_DOMAIN}/userinfo`, {
				headers: { Authorization: `Bearer ${tokens.access_token}` },
				signal: AbortSignal.timeout(5000),
			});
			if (userinfoRes.ok) {
				const info = (await userinfoRes.json()) as Record<string, unknown>;
				if (!name)
					name = (info.name as string) ?? (info.nickname as string) ?? "User";
				if (!email) email = (info.email as string) ?? "";
			}
		}

		try {
			await ensureUser(hub, { name, email }, 10_000);
		} catch (e: unknown) {
			if (e instanceof HubApiError) {
				if (e.status === 409) {
					throw error(409, AUTH_ERROR_DETAILS.accountConflict);
				}
				if (e.status === 400) {
					throw error(400, AUTH_ERROR_DETAILS.invalidProfile);
				}
				throw error(502, AUTH_ERROR_DETAILS.provisionFailed);
			}

			if (isRedirect(e) || isHttpError(e)) throw e;
			console.error("ensure-user call failed: ", e);
			throw error(502, AUTH_ERROR_DETAILS.provisionFailed);
		}

		let role = "";
		let officeIds: string[] = [];
		let currentOfficeId: string | null = null;
		try {
			const me = await getMeContext(hub, 5_000);
			role = me.role;
			officeIds = me.office_ids;
			currentOfficeId = me.current_office_id;
		} catch (e: unknown) {
			console.error("/me failed: ", e);
			throw error(502, AUTH_ERROR_DETAILS.userRoleLoadFailed);
		}

		const expiresAt =
			Math.floor(Date.now() / 1000) + (tokens.expires_in ?? 3600);
		const encryptionKey = await deriveKey(SESSION_SECRET);

		const session = await new EncryptJWT({
			access_token: tokens.access_token,
			refresh_token: tokens.refresh_token,
			id_token: tokens.id_token,
			expires_at: expiresAt,
			role,
			name,
			email,
			office_ids: officeIds,
			current_office_id: currentOfficeId,
		})
			.setProtectedHeader({ alg: "dir", enc: "A256GCM" })
			.setIssuedAt()
			.setExpirationTime("7d")
			.encrypt(encryptionKey);

		cookies.set("lp_session", session, {
			path: "/",
			httpOnly: true,
			sameSite: "lax",
			secure: url.protocol === "https:",
			maxAge: 60 * 60 * 24 * 7,
		});

		throw redirect(302, safeRedirectPath(state));
	} catch (e: unknown) {
		if (isRedirect(e)) throw e;

		if (isHttpError(e)) {
			const detail =
				typeof e.body?.message === "string"
					? e.body.message
					: AUTH_ERROR_DETAILS.unknown;
			throw redirect(303, authErrorRedirectPath(lang, e.status, detail));
		}

		console.error("Unhandled callback failure:", e);
		throw redirect(
			303,
			authErrorRedirectPath(lang, 500, AUTH_ERROR_DETAILS.unknown),
		);
	}
};
