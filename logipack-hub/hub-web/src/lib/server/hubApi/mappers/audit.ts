import type {
	AuditEventDto,
	AuditPageDto,
	ListAuditResponseDto,
} from "../dto/audit";
import {
	HubApiMappingError,
	cleanNullableString,
	requireIsoDateTime,
	requireRecord,
	requireString,
} from "../normalizers";

export type AuditEvent = {
	id: string;
	occurred_at: string;
	actor_user_id: string | null;
	actor_display_name: string | null;
	action_key: string;
	entity_type: string | null;
	entity_id: string | null;
	entity_label: string | null;
	office_id: string | null;
	office_label: string | null;
	target_route: string | null;
	metadata: Record<string, unknown> | null;
};

export type AuditPage = {
	limit: number;
	next_cursor: string | null;
	has_next: boolean;
};

export type AuditListResult = {
	events: AuditEvent[];
	page: AuditPage;
};

function requireBoolean(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): boolean {
	if (typeof args.value !== "boolean") {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected boolean for ${args.field}`,
		});
	}

	return args.value;
}

function requireNumber(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): number {
	if (typeof args.value !== "number" || !Number.isFinite(args.value)) {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected finite number for ${args.field}`,
		});
	}

	return args.value;
}

function mapAuditEventDto(dto: AuditEventDto): AuditEvent {
	const endpoint = "GET /admin/audit";
	const obj = requireRecord({ endpoint, field: "event", value: dto });

	const metadataValue = obj.metadata;
	const metadata =
		metadataValue === undefined || metadataValue === null
			? null
			: requireRecord({
					endpoint,
					field: "event.metadata",
					value: metadataValue,
				});

	return {
		id: requireString({
			endpoint,
			field: "event.id",
			value: obj.id,
			nonEmpty: true,
		}),
		occurred_at: requireIsoDateTime({
			endpoint,
			field: "event.occurred_at",
			value: obj.occurred_at,
		}),
		actor_user_id: cleanNullableString({
			endpoint,
			field: "event.actor_user_id",
			value: obj.actor_user_id,
		}),
		actor_display_name: cleanNullableString({
			endpoint,
			field: "event.actor_display_name",
			value: obj.actor_display_name,
		}),
		action_key: requireString({
			endpoint,
			field: "event.action_key",
			value: obj.action_key,
			nonEmpty: true,
		}),
		entity_type: cleanNullableString({
			endpoint,
			field: "event.entity_type",
			value: obj.entity_type,
		}),
		entity_id: cleanNullableString({
			endpoint,
			field: "event.entity_id",
			value: obj.entity_id,
		}),
		entity_label: cleanNullableString({
			endpoint,
			field: "event.entity_label",
			value: obj.entity_label,
		}),
		office_id: cleanNullableString({
			endpoint,
			field: "event.office_id",
			value: obj.office_id,
		}),
		office_label: cleanNullableString({
			endpoint,
			field: "event.office_label",
			value: obj.office_label,
		}),
		target_route: cleanNullableString({
			endpoint,
			field: "event.target_route",
			value: obj.target_route,
		}),
		metadata,
	};
}

function mapAuditPageDto(dto: AuditPageDto): AuditPage {
	const endpoint = "GET /admin/audit";
	const obj = requireRecord({ endpoint, field: "page", value: dto });

	return {
		limit: requireNumber({
			endpoint,
			field: "page.limit",
			value: obj.limit,
		}),
		next_cursor: cleanNullableString({
			endpoint,
			field: "page.next_cursor",
			value: obj.next_cursor,
		}),
		has_next: requireBoolean({
			endpoint,
			field: "page.has_next",
			value: obj.has_next,
		}),
	};
}

export function mapListAuditResponseDto(
	dto: ListAuditResponseDto,
): AuditListResult {
	const endpoint = "GET /admin/audit";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.events)) {
		throw new HubApiMappingError({
			endpoint,
			field: "events",
			message: "expected events[]",
		});
	}

	return {
		events: obj.events.map((event, index) =>
			mapAuditEventDto(
				requireRecord({
					endpoint,
					field: `events[${index}]`,
					value: event,
				}) as AuditEventDto,
			),
		),
		page: mapAuditPageDto(
			requireRecord({
				endpoint,
				field: "page",
				value: obj.page,
			}) as AuditPageDto,
		),
	};
}
