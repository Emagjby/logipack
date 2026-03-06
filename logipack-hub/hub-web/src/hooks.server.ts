import type { Handle } from "@sveltejs/kit";
import { parseSession, type LpSession } from "$lib/server/session.server";
import { jwtDecrypt, EncryptJWT } from "jose";
import {
	AUTH0_DOMAIN,
	AUTH0_CLIENT_ID,
	AUTH0_CLIENT_SECRET,
	AUTH0_AUDIENCE,
	SESSION_SECRET,
} from "$env/static/private";

const SUPPORTED = ["en", "bg"] as const;
type Lang = (typeof SUPPORTED)[number];

const enc = new TextEncoder();

async function deriveKey(secret: string): Promise<Uint8Array> {
	const data = enc.encode(secret);
	const hashBuffer = await crypto.subtle.digest("SHA-256", data);
	return new Uint8Array(hashBuffer);
}

function parseAcceptLanguage(header: string | null): string[] {
	if (!header) return [];
	return header
		.split(",")
		.map((part) => part.split(";")[0]?.trim().toLowerCase())
		.filter(Boolean);
}

function pickSupported(preferred: string[]): Lang | null {
	for (const raw of preferred) {
		const base = raw.split("-")[0];
		if (SUPPORTED.includes(base as Lang)) return base as Lang;
	}
	return null;
}

function setLangCookie(event: Parameters<Handle>[0]["event"], lang: Lang) {
	if (event.cookies.get("lang") !== lang) {
		event.cookies.set("lang", lang, {
			path: "/",
			sameSite: "lax",
			httpOnly: false,
			secure: event.url.protocol === "https:",
			maxAge: 60 * 60 * 24 * 365,
		});
	}
}

function isAuthRoute(pathname: string) {
	return pathname === "/callback" || pathname === "/logout";
}

// TODO
async function refreshAccessToken(refreshToken: string) {
	const res = await fetch(`https://${AUTH0_DOMAIN}/oauth/token`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		signal: AbortSignal.timeout(5000),
		body: JSON.stringify({
			grant_type: "refresh_token",
			client_id: AUTH0_CLIENT_ID,
			client_secret: AUTH0_CLIENT_SECRET,
			refresh_token: refreshToken,
			audience: AUTH0_AUDIENCE,
		}),
	});

	if (!res.ok) return null;

	return (await res.json()) as {
		access_token: string;
		expires_in: number;
		id_token?: string;
		refresh_token?: string;
		token_type: string;
	};
}

async function signSession(payload: Record<string, unknown>, key: Uint8Array) {
	return new EncryptJWT(payload)
		.setProtectedHeader({ alg: "dir", enc: "A256GCM" })
		.setIssuedAt()
		.setExpirationTime("7d")
		.encrypt(key);
}

function isBypassPath(pathname: string) {
	return (
		pathname.startsWith("/_app") ||
		pathname.startsWith("/@") ||
		pathname.startsWith("/favicon") ||
		pathname === "/robots.txt" ||
		pathname.startsWith("/sitemap") ||
		pathname.startsWith("/manifest") ||
		pathname.startsWith("/assets") ||
		pathname.startsWith("/fonts") ||
		pathname.startsWith("/images")
	);
}

export const handle: Handle = async ({ event, resolve }) => {
	const { url, cookies, request } = event;
	const encryptionKey = await deriveKey(SESSION_SECRET);

	const seg = url.pathname.split("/")[1];

	if (isBypassPath(url.pathname)) {
		return resolve(event);
	}

	if (SUPPORTED.includes(seg as Lang)) {
		const lang = seg as Lang;
		event.locals.lang = lang;
		setLangCookie(event, lang);
	} else {
		const cookieLang = cookies.get("lang");
		const accept = pickSupported(
			parseAcceptLanguage(request.headers.get("accept-language")),
		);

		const lang =
			(SUPPORTED.includes(cookieLang as Lang) ? (cookieLang as Lang) : null) ??
			accept ??
			"en";

		if (!isAuthRoute(url.pathname)) {
			if (url.pathname === "/") {
				return new Response(null, {
					status: 302,
					headers: { location: `/${lang}${url.search}` },
				});
			}

			return new Response(null, {
				status: 302,
				headers: { location: `/${lang}${url.pathname}${url.search}` },
			});
		}

		event.locals.lang = lang;
		setLangCookie(event, lang);
	}

	event.locals.session = null;

	const raw = cookies.get("lp_session");
	if (raw) {
		try {
			const { payload } = await jwtDecrypt(raw, encryptionKey);
			let session = parseSession(payload);

			if (!session) {
				cookies.delete("lp_session", { path: "/" });
				event.locals.session = null;
				return resolve(event);
			}

			const now = Math.floor(Date.now() / 1000);
			const expiresAt = Number(session.expires_at ?? 0);

			const rt = session.refresh_token;

			const shouldRefresh =
				typeof rt === "string" &&
				rt.trim().length > 0 &&
				expiresAt > 0 &&
				expiresAt - now < 30;

			if (shouldRefresh) {
				const refreshed = await refreshAccessToken(rt);
				if (refreshed?.access_token) {
					const newExpiresAt = now + (refreshed.expires_in ?? 3600);

					const nextPayload = {
						...session,
						access_token: refreshed.access_token,
						expires_at: newExpiresAt,
						id_token: refreshed.id_token ?? session.id_token,
						refresh_token: refreshed.refresh_token ?? session.refresh_token,
						office_ids: session.office_ids,
						current_office_id: session.current_office_id,
					};

					const parsed = parseSession(nextPayload);
					if (!parsed) {
						cookies.delete("lp_session", { path: "/" });
						event.locals.session = null;
						return resolve(event);
					}

					session = parsed;

					const newJwt = await signSession(nextPayload, encryptionKey);

					cookies.set("lp_session", newJwt, {
						path: "/",
						httpOnly: true,
						sameSite: "lax",
						secure: url.protocol === "https:",
						maxAge: 60 * 60 * 24 * 7,
					});
				} else {
					cookies.delete("lp_session", { path: "/" });
					event.locals.session = null;
					return resolve(event);
				}
			}

			event.locals.session = session;
		} catch {
			cookies.delete("lp_session", { path: "/" });
		}
	}

	return resolve(event);
};
