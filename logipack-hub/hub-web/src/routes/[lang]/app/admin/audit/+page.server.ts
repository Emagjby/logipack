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
			totalCount: number | null;
			nextCursor: string | null;
			hasNext: boolean;
	  }
	| {
			state: "empty";
			events: AuditEvent[];
			limit: number;
			totalCount: number | null;
			nextCursor: null;
			hasNext: false;
	  }
	| {
			state: "error";
			events: AuditEvent[];
			limit: number;
			totalCount: number | null;
			nextCursor: null;
			hasNext: false;
	  };

type AuditFilters = {
	actor: string | null;
	entityType: string | null;
	action: string | null;
	from: string | null;
	to: string | null;
};

function parseLimit(raw: string | null): number {
	const parsed = Number.parseInt(raw ?? "", 10);
	if (!Number.isFinite(parsed)) {
		return 10;
	}

	return Math.min(100, Math.max(1, parsed));
}

function parsePage(raw: string | null): number {
	const parsed = Number.parseInt(raw ?? "", 10);
	if (!Number.isFinite(parsed)) return 1;
	return Math.max(1, parsed);
}

function cleanFilter(raw: string | null): string | null {
	const trimmed = raw?.trim() ?? "";
	return trimmed.length > 0 ? trimmed : null;
}

function readFilters(url: URL): AuditFilters {
	return {
		actor: cleanFilter(url.searchParams.get("actor")),
		entityType: cleanFilter(url.searchParams.get("entity_type")),
		action: cleanFilter(url.searchParams.get("action")),
		from: cleanFilter(url.searchParams.get("from")),
		to: cleanFilter(url.searchParams.get("to")),
	};
}

function buildPageHref(url: URL, page: number, limit: number): string {
	const pageUrl = new URL(url);
	pageUrl.searchParams.delete("cursor");
	pageUrl.searchParams.delete("prev");
	pageUrl.searchParams.set("page", String(page));
	pageUrl.searchParams.set("limit", String(limit));

	const query = pageUrl.searchParams.toString();
	return query ? `${pageUrl.pathname}?${query}` : pageUrl.pathname;
}

function buildPaginationPages(currentPage: number, totalPages: number | null): number[] {
	if (totalPages === null) return [currentPage];

	const start = Math.max(1, Math.min(currentPage - 2, totalPages - 4));
	const end = Math.min(totalPages, start + 4);
	return Array.from({ length: end - start + 1 }, (_, index) => start + index);
}

export const load: PageServerLoad = async ({ url, fetch, locals }) => {
	const limit = parseLimit(url.searchParams.get("limit"));
	const cursor = url.searchParams.get("cursor")?.trim() || null;
	const currentPage = cursor ? 1 : parsePage(url.searchParams.get("page"));
	const filters = readFilters(url);

	try {
		const client = createHubApiClient({
			fetch,
			locals,
			baseUrl: HUB_API_BASE,
		});

		const audit = await listAuditEvents(client, {
			limit,
			page: currentPage,
			cursor,
			actor: filters.actor,
			entityType: filters.entityType,
			action: filters.action,
			from: filters.from,
			to: filters.to,
		});
		const totalCount = audit.page.total_count;
		const totalPages = totalCount === null ? null : Math.max(1, Math.ceil(totalCount / audit.page.limit));
		const normalizedPage = totalPages === null ? currentPage : Math.min(currentPage, totalPages);
		const result: AdminAuditResult =
			audit.events.length > 0
				? {
						state: "ok",
						events: audit.events,
						limit: audit.page.limit,
						totalCount,
						nextCursor: audit.page.next_cursor,
						hasNext: audit.page.has_next,
					}
				: {
						state: "empty",
						events: [],
						limit: audit.page.limit,
						totalCount,
						nextCursor: null,
						hasNext: false,
					};

		return {
			result,
			filters,
			currentPage: normalizedPage,
			totalPages,
			paginationPages: buildPaginationPages(normalizedPage, totalPages),
			previousPageHref:
				normalizedPage > 1
					? buildPageHref(url, normalizedPage - 1, audit.page.limit)
					: null,
			nextPageHref:
				totalPages !== null && normalizedPage < totalPages
					? buildPageHref(url, normalizedPage + 1, audit.page.limit)
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
				totalCount: null,
				nextCursor: null,
				hasNext: false,
			},
			filters,
			currentPage,
			totalPages: null,
			paginationPages: [currentPage],
			previousPageHref: null,
			nextPageHref: null,
		};
	}
};
