import { HUB_API_BASE } from "$env/static/private";
import {
	type AuditEvent,
	HubApiError,
	createHubApiClient,
	listAuditEvents,
} from "$lib/server/hubApi";
import type { PageServerLoad } from "./$types";

type AdminAuditResult =
	| {
			state: "ok";
			events: AuditEvent[];
			limit: number;
			nextCursor: string | null;
			hasNext: boolean;
	  }
	| {
			state: "empty";
			events: AuditEvent[];
			limit: number;
			nextCursor: null;
			hasNext: false;
	  }
	| {
			state: "error";
			events: AuditEvent[];
			limit: number;
			nextCursor: null;
			hasNext: false;
	  };

function parseLimit(raw: string | null): number {
	const parsed = Number.parseInt(raw ?? "", 10);
	if (!Number.isFinite(parsed)) {
		return 10;
	}

	return Math.min(100, Math.max(1, parsed));
}

function buildNextPageHref(url: URL, nextCursor: string, limit: number): string {
	const nextUrl = new URL(url);
	nextUrl.searchParams.set("cursor", nextCursor);
	nextUrl.searchParams.set("limit", String(limit));

	const query = nextUrl.searchParams.toString();
	return query ? `${nextUrl.pathname}?${query}` : nextUrl.pathname;
}

export const load: PageServerLoad = async ({ url, fetch, locals }) => {
	const limit = parseLimit(url.searchParams.get("limit"));
	const cursor = url.searchParams.get("cursor")?.trim() || null;

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const audit = await listAuditEvents(client, { limit, cursor });
		const result: AdminAuditResult =
			audit.events.length > 0
				? {
						state: "ok",
						events: audit.events,
						limit: audit.page.limit,
						nextCursor: audit.page.next_cursor,
						hasNext: audit.page.has_next,
					}
				: {
						state: "empty",
						events: [],
						limit: audit.page.limit,
						nextCursor: null,
						hasNext: false,
					};

		return {
			result,
			nextPageHref:
				audit.page.has_next && audit.page.next_cursor
					? buildNextPageHref(url, audit.page.next_cursor, audit.page.limit)
					: null,
		};
	} catch (error) {
		if (error instanceof HubApiError) {
			console.error("admin.audit.list failed", {
				status: error.status,
				code: error.code,
				message: error.message,
				upstream: error.upstream,
			});
		} else {
			console.error("admin.audit.list failed", error);
		}

		return {
			result: {
				state: "error" as const,
				events: [],
				limit,
				nextCursor: null,
				hasNext: false,
			},
			nextPageHref: null,
		};
	}
};
