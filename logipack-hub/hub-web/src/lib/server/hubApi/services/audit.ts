import type { HubApiClient } from "../index";
import type { ListAuditResponseDto } from "../dto/audit";
import type { AuditListResult } from "../mappers/audit";
import { mapListAuditResponseDto } from "../mappers/audit";

export async function listAuditEvents(
	client: HubApiClient,
	args?: {
		limit?: number;
		page?: number;
		cursor?: string | null;
		actor?: string | null;
		entityType?: string | null;
		action?: string | null;
		from?: string | null;
		to?: string | null;
	},
	timeoutMs = 10_000,
): Promise<AuditListResult> {
	const query = new URLSearchParams();

	if (typeof args?.limit === "number" && Number.isFinite(args.limit)) {
		query.set("limit", String(args.limit));
	}

	if (typeof args?.page === "number" && Number.isFinite(args.page)) {
		query.set("page", String(args.page));
	}

	if (args?.cursor) {
		query.set("cursor", args.cursor);
	}

	if (args?.actor) {
		query.set("actor", args.actor);
	}

	if (args?.entityType) {
		query.set("entity_type", args.entityType);
	}

	if (args?.action) {
		query.set("action", args.action);
	}

	if (args?.from) {
		query.set("from", args.from);
	}

	if (args?.to) {
		query.set("to", args.to);
	}

	const suffix = query.size > 0 ? `?${query.toString()}` : "";
	const res = await client.get<ListAuditResponseDto>(`/admin/audit${suffix}`, {
		timeoutMs,
	});

	return mapListAuditResponseDto(res.data);
}
