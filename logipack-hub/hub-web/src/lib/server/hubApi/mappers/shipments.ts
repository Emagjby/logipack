import type {
	ShipmentListItemDto,
	ShipmentDetailDto,
	ShipmentTimelineItemDto,
	ListShipmentsResponseDto,
	GetShipmentResponseDto,
	GetShipmentTimelineResponseDto,
	CreateShipmentResponseDto,
	ChangeShipmentStatusRequestDto,
} from "../dto/shipments";
import {
	requireIsoDateTime,
	requireRecord,
	requireString,
	cleanNullableString,
	HubApiMappingError,
} from "../normalizers";
import {
	normalizeShipmentStatus,
	type ShipmentStatus,
} from "$lib/domain/shipmentStatus";
import type { ShipmentRow } from "$lib/domain/shipmentStatus";
import type { StrataPackage } from "$lib/domain/strataPackage";

// ── App-safe shapes ─────────────────────────────────────────────────

export type ShipmentListItem = ShipmentRow;

export type ShipmentDetail = {
	id: string;
	client_id: string;
	current_status: ShipmentStatus | "unknown";
	current_office_id: string | null;
	created_at: string;
	updated_at: string;
};

export type ShipmentTimelineItem = {
	seq: number;
	event_type: string;
	created_at: string;
	scb: string;
	payload: Record<string, unknown> | null;
};

export type StatusHistoryRow = {
	id: string;
	from_status: ShipmentStatus | "unknown" | null;
	to_status: ShipmentStatus | "unknown";
	changed_at: string;
	actor_user_id: string | null;
	office_id: string | null;
	notes: string | null;
};

// ── Helpers ─────────────────────────────────────────────────────────

function requireNumber(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): number {
	const { endpoint, field, value } = args;
	if (typeof value !== "number" || !Number.isFinite(value)) {
		throw new HubApiMappingError({
			endpoint,
			field,
			message: `expected finite number for ${field}`,
		});
	}
	return value;
}

/**
 * Deterministic hash derived from a seed string.
 * Uses FNV-1a-like mixing to produce a 32-hex-char string.
 */
function deterministicHash(seed: string): string {
	let h = 0x811c9dc5;
	for (let i = 0; i < seed.length; i++) {
		h ^= seed.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	const a = (h >>> 0).toString(16).padStart(8, "0");
	const b = ((h ^ 0xdeadbeef) >>> 0).toString(16).padStart(8, "0");
	const c = ((h ^ 0xcafebabe) >>> 0).toString(16).padStart(8, "0");
	const d = ((h ^ 0xfeedface) >>> 0).toString(16).padStart(8, "0");
	return `${a}${b}${c}${d}`;
}

// ── List mappers ────────────────────────────────────────────────────

export function mapShipmentListItemDtoToShipmentListItem(
	dto: ShipmentListItemDto,
): ShipmentListItem {
	const endpoint = "GET /shipments";
	const obj = requireRecord({ endpoint, field: "shipment", value: dto });

	const id = requireString({
		endpoint,
		field: "shipment.id",
		value: obj.id,
		nonEmpty: true,
	});

	requireString({
		endpoint,
		field: "shipment.client_id",
		value: obj.client_id,
		nonEmpty: true,
	});

	const rawStatus = requireString({
		endpoint,
		field: "shipment.current_status",
		value: obj.current_status,
	});

	const status = normalizeShipmentStatus(rawStatus);

	const office =
		obj.current_office_id === undefined || obj.current_office_id === null
			? "—"
			: requireString({
					endpoint,
					field: "shipment.current_office_id",
					value: obj.current_office_id,
				}) || "—";

	const updatedAt = requireIsoDateTime({
		endpoint,
		field: "shipment.updated_at",
		value: obj.updated_at,
	});

	return { id, status, office, updatedAt };
}

export function mapListShipmentsResponseDto(
	dto: ListShipmentsResponseDto,
): ShipmentListItem[] {
	const endpoint = "GET /shipments";
	if (Array.isArray(dto)) {
		return dto.map((s, i) =>
			mapShipmentListItemDtoToShipmentListItem(
				requireRecord({
					endpoint,
					field: `shipments[${i}]`,
					value: s,
				}) as ShipmentListItemDto,
			),
		);
	}

	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.shipments)) {
		throw new HubApiMappingError({
			endpoint,
			field: "shipments",
			message: "expected shipments[]",
		});
	}

	return obj.shipments.map((s: unknown, i: number) =>
		mapShipmentListItemDtoToShipmentListItem(
			requireRecord({
				endpoint,
				field: `shipments[${i}]`,
				value: s,
			}) as ShipmentListItemDto,
		),
	);
}

// ── Detail mapper ───────────────────────────────────────────────────

export function mapShipmentDetailDtoToShipmentDetail(
	dto: ShipmentDetailDto,
): ShipmentDetail {
	const endpoint = "GET /shipments/:id";
	const obj = requireRecord({ endpoint, field: "shipment", value: dto });

	const id = requireString({
		endpoint,
		field: "shipment.id",
		value: obj.id,
		nonEmpty: true,
	});

	const client_id =
		typeof obj.client_id === "string" && obj.client_id.trim()
			? obj.client_id.trim()
			: requireString({
					endpoint,
					field: "shipment.client.id",
					value: requireRecord({
						endpoint,
						field: "shipment.client",
						value: obj.client,
					}).id,
					nonEmpty: true,
				});

	const rawStatus = requireString({
		endpoint,
		field: "shipment.current_status",
		value: obj.current_status,
	});

	const current_status = normalizeShipmentStatus(rawStatus);

	const current_office_id =
		obj.current_office_id !== undefined
			? cleanNullableString({
					endpoint,
					field: "shipment.current_office_id",
					value: obj.current_office_id,
				})
			: obj.current_office
				? cleanNullableString({
						endpoint,
						field: "shipment.current_office.id",
						value: requireRecord({
							endpoint,
							field: "shipment.current_office",
							value: obj.current_office,
						}).id,
					})
				: null;

	const created_at = requireIsoDateTime({
		endpoint,
		field: "shipment.created_at",
		value: obj.created_at,
	});

	const updated_at = requireIsoDateTime({
		endpoint,
		field: "shipment.updated_at",
		value: obj.updated_at,
	});

	return {
		id,
		client_id,
		current_status,
		current_office_id,
		created_at,
		updated_at,
	};
}

export function mapGetShipmentResponseDto(
	dto: GetShipmentResponseDto,
): ShipmentDetail {
	const endpoint = "GET /shipments/:id";
	if (dto && typeof dto === "object" && !Array.isArray(dto)) {
		return mapShipmentDetailDtoToShipmentDetail(
			requireRecord({
				endpoint,
				field: "shipment",
				value: dto,
			}) as ShipmentDetailDto,
		);
	}

	throw new HubApiMappingError({
		endpoint,
		field: "response",
		message: "expected shipment object",
	});
}

// ── Timeline mapper ─────────────────────────────────────────────────

export function mapShipmentTimelineItemDto(
	dto: ShipmentTimelineItemDto,
	index: number,
): ShipmentTimelineItem {
	const endpoint = "GET /shipments/:id/timeline";
	const obj = requireRecord({
		endpoint,
		field: `timeline[${index}]`,
		value: dto,
	});

	const seq = requireNumber({
		endpoint,
		field: `timeline[${index}].seq`,
		value: obj.seq,
	});

	const event_type = requireString({
		endpoint,
		field: `timeline[${index}].event_type`,
		value: obj.event_type,
		nonEmpty: true,
	});

	const created_at =
		obj.created_at === undefined || obj.created_at === null
			? new Date().toISOString()
			: requireIsoDateTime({
					endpoint,
					field: `timeline[${index}].created_at`,
					value: obj.created_at,
				});

	const scb = requireString({
		endpoint,
		field: `timeline[${index}].scb`,
		value: obj.scb,
		nonEmpty: true,
	});

	const payload =
		obj.payload && typeof obj.payload === "object" ? (obj.payload as Record<string, unknown>) : null;

	return { seq, event_type, created_at, scb, payload };
}

export function mapGetShipmentTimelineResponseDto(
	dto: GetShipmentTimelineResponseDto,
): ShipmentTimelineItem[] {
	const endpoint = "GET /shipments/:id/timeline";
	if (Array.isArray(dto)) {
		return dto.map((t, i) =>
			mapShipmentTimelineItemDto(
				requireRecord({
					endpoint,
					field: `timeline[${i}]`,
					value: t,
				}) as ShipmentTimelineItemDto,
				i,
			),
		);
	}

	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.timeline)) {
		throw new HubApiMappingError({
			endpoint,
			field: "timeline",
			message: "expected timeline[]",
		});
	}

	return obj.timeline.map((t: unknown, i: number) =>
		mapShipmentTimelineItemDto(
			requireRecord({
				endpoint,
				field: `timeline[${i}]`,
				value: t,
			}) as ShipmentTimelineItemDto,
			i,
		),
	);
}

// ── Create response mapper ──────────────────────────────────────────

export function mapCreateShipmentResponseDto(
	dto: CreateShipmentResponseDto,
): { id: string } {
	const endpoint = "POST /shipments";
	const obj = requireRecord({ endpoint, field: "response", value: dto });
	const id =
		typeof obj.shipment_id === "string"
			? obj.shipment_id
			: obj.shipment && typeof obj.shipment === "object"
				? requireString({
						endpoint,
						field: "shipment.id",
						value: (obj.shipment as Record<string, unknown>).id,
						nonEmpty: true,
					})
				: null;

	if (!id) {
		throw new HubApiMappingError({
			endpoint,
			field: "shipment_id",
			message: "expected shipment_id",
		});
	}

	return { id };
}

// ── Projection: timeline + detail → statusHistory ───────────────────

/**
 * Build StatusHistoryRow[] from backend timeline events and shipment detail.
 * Each timeline event that represents a status transition produces a row.
 * Events that don't look like status changes are still included with best-effort mapping.
 */
export function buildStatusHistory(
	shipmentId: string,
	detail: ShipmentDetail,
	timeline: ShipmentTimelineItem[],
): StatusHistoryRow[] {
	const sorted = [...timeline].sort((a, b) => a.seq - b.seq);
	let lastKnownOfficeId: string | null = detail.current_office_id;

	return sorted.flatMap((event, index) => {
		const extracted = parseStatusHistoryPayload(event.payload);
		const isObsoleteShipmentEvent =
			event.seq === 1 && event.event_type.trim().toLowerCase() === "shipment";
		if (isObsoleteShipmentEvent) {
			return [];
		}

		const hasExplicitStatus =
			extracted.from_status !== null || extracted.to_status !== null;
		const hasContext =
			extracted.actor_user_id !== null ||
			extracted.office_id !== null ||
			extracted.notes !== null;
		if (!hasExplicitStatus && !hasContext) {
			return [];
		}

		const from_status = extracted.from_status;
		const to_status =
			extracted.to_status ??
			(index === sorted.length - 1 ? detail.current_status : "unknown");
		const actor_user_id = extracted.actor_user_id;
		const office_id = extracted.office_id ?? lastKnownOfficeId;
		if (office_id) {
			lastKnownOfficeId = office_id;
		}
		const notes = extracted.notes;

		return [{
			id: `sh-${shipmentId}-${index + 1}`,
			from_status,
			to_status,
			changed_at: extracted.changed_at ?? event.created_at,
			actor_user_id,
			office_id,
			notes,
		}];
	});
}

export function deriveCurrentOfficeIdFromTimeline(
	timeline: ShipmentTimelineItem[],
	fallbackOfficeId: string | null,
): string | null {
	if (timeline.length === 0) return fallbackOfficeId;

	const sortedDesc = [...timeline].sort((a, b) => b.seq - a.seq);
	const lastEvent = sortedDesc[0];
	if (!lastEvent) return fallbackOfficeId;

	const extracted = parseStatusHistoryPayload(lastEvent.payload);
	return extracted.office_id ?? fallbackOfficeId;
}

// ── Projection: timeline + detail → StrataPackage[] ─────────────────

/**
 * Build StrataPackage[] from backend timeline events.
 * Each package hash is deterministically derived from shipmentId + seq.
 */
export function buildStrataPackages(
	shipmentId: string,
	timeline: ShipmentTimelineItem[],
): StrataPackage[] {
	const sorted = [...timeline].sort((a, b) => a.seq - b.seq);
	const streamId = `stream-shipment-${shipmentId}`;

	return sorted.map((event, index) => {
		const hash = deterministicHash(`${shipmentId}-pkg-${event.seq}`);
		const prevHash =
			index === 0
				? null
				: deterministicHash(
						`${shipmentId}-pkg-${sorted[index - 1].seq}`,
					);

		const payload: Record<string, unknown> =
			event.payload ?? {
				event_type: event.event_type,
				shipment_id: shipmentId,
				timestamp: event.created_at,
				raw_scb_base64: event.scb,
			};

		return {
			hash,
			prev_hash: prevHash,
			stream_id: streamId,
			seq: event.seq,
			event_type: event.event_type,
			created_at: event.created_at,
			payload_json: payload,
		};
	});
}

// ── Change status mapper ────────────────────────────────────────────

/**
 * Map frontend ShipmentStatus to backend SCREAMING_SNAKE_CASE.
 * The Rust ShipmentStatus enum uses serde(rename_all = "SCREAMING_SNAKE_CASE"):
 * New -> NEW, Accepted -> ACCEPTED, Processed -> PROCESSED,
 * InTransit -> IN_TRANSIT, Delivered -> DELIVERED, Cancelled -> CANCELLED.
 *
 * Frontend uses lowercase/snake_case: "new", "accepted", "pending" (alias for Processed),
 * "in_transit", "delivered", "cancelled".
 */
const STATUS_TO_BACKEND: Record<string, string> = {
	new: "NEW",
	accepted: "ACCEPTED",
	pending: "PROCESSED",
	processed: "PROCESSED",
	in_transit: "IN_TRANSIT",
	delivered: "DELIVERED",
	cancelled: "CANCELLED",
};

export function mapStatusToBackend(frontendStatus: string): string {
	const mapped = STATUS_TO_BACKEND[frontendStatus.toLowerCase()];
	if (!mapped) {
		throw new HubApiMappingError({
			endpoint: "POST /shipments/:id/status",
			field: "to_status",
			message: `unknown frontend status: "${frontendStatus}"`,
		});
	}
	return mapped;
}

export function buildChangeStatusRequestDto(input: {
	to_status: string;
	to_office_id?: string | null;
	notes?: string | null;
}): ChangeShipmentStatusRequestDto {
	const endpoint = "POST /shipments/:id/status";

	const to_status = mapStatusToBackend(
		requireString({
			endpoint,
			field: "to_status",
			value: input.to_status,
			nonEmpty: true,
		}),
	);

	const to_office_id = input.to_office_id
		? cleanNullableString({
				endpoint,
				field: "to_office_id",
				value: input.to_office_id,
			})
		: null;

	const notes = input.notes
		? cleanNullableString({
				endpoint,
				field: "notes",
				value: input.notes,
			})
		: null;

	return { to_status, to_office_id, notes };
}
function statusFromPayload(value: unknown): ShipmentStatus | "unknown" | null {
	if (!value || typeof value !== "string") return null;
	return normalizeShipmentStatus(value);
}

function stringFromPayload(value: unknown): string | null {
	return typeof value === "string" && value.trim() ? value.trim() : null;
}

function parseStatusHistoryPayload(value: unknown): {
	from_status: ShipmentStatus | "unknown" | null;
	to_status: ShipmentStatus | "unknown" | null;
	actor_user_id: string | null;
	office_id: string | null;
	notes: string | null;
	changed_at: string | null;
} {
	if (!value) {
		return {
			from_status: null,
			to_status: null,
			actor_user_id: null,
			office_id: null,
			notes: null,
			changed_at: null,
		};
	}

	let payload: Record<string, unknown> | null = null;

	if (Array.isArray(value)) {
		const tuplePayload = value[1];
		if (tuplePayload && typeof tuplePayload === "object") {
			payload = tuplePayload as Record<string, unknown>;
		}
	} else if (typeof value === "object") {
		payload = value as Record<string, unknown>;
	}

	if (!payload) {
		return {
			from_status: null,
			to_status: null,
			actor_user_id: null,
			office_id: null,
			notes: null,
			changed_at: null,
		};
	}

	const occurredRaw = payload.occured_at ?? payload.occurred_at;
	const occurredAt =
		typeof occurredRaw === "number" && Number.isFinite(occurredRaw)
			? new Date(occurredRaw).toISOString()
			: typeof occurredRaw === "string" && Number.isFinite(Date.parse(occurredRaw))
				? new Date(occurredRaw).toISOString()
				: null;

	const toStatus = statusFromPayload(payload.to_status ?? payload.status);

	return {
		from_status: statusFromPayload(payload.from_status),
		to_status: toStatus,
		actor_user_id: stringFromPayload(payload.actor_user_id),
		office_id: stringFromPayload(payload.to_office_id ?? payload.office_id),
		notes: stringFromPayload(payload.notes),
		changed_at: occurredAt,
	};
}
