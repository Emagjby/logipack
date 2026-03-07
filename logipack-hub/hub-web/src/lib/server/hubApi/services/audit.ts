import type { HubApiClient } from "../index";
import type { ListAuditResponseDto } from "../dto/audit";
import type { AuditListResult } from "../mappers/audit";
import { mapListAuditResponseDto } from "../mappers/audit";

export async function listAuditEvents(
	client: HubApiClient,
	args?: {
		limit?: number;
		cursor?: string | null;
	},
	timeoutMs = 10_000,
): Promise<AuditListResult> {
	const query = new URLSearchParams();

	if (typeof args?.limit === "number" && Number.isFinite(args.limit)) {
		query.set("limit", String(args.limit));
	}

	if (args?.cursor) {
		query.set("cursor", args.cursor);
	}

	const suffix = query.size > 0 ? `?${query.toString()}` : "";
	const res = await client.get<ListAuditResponseDto>(`/admin/audit${suffix}`, {
		timeoutMs,
	});

	return mapListAuditResponseDto(res.data);
}
