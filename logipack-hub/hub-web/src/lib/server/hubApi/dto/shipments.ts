// ── List ─────────────────────────────────────────────────────────────

export type ShipmentListItemDto = {
	id: string;
	client_id: string;
	current_status: string;
	current_office_id?: string | null;
	created_at: string;
	updated_at: string;
};

export type ListShipmentsResponseDto = ShipmentListItemDto[] | {
	shipments: ShipmentListItemDto[];
};

// ── Detail ───────────────────────────────────────────────────────────

export type ShipmentDetailDto = {
	id: string;
	client?: { id: string } | null;
	client_id?: string | null;
	current_status: string;
	current_office?: { id: string } | null;
	current_office_id?: string | null;
	created_at: string;
	updated_at: string;
};

export type GetShipmentResponseDto = ShipmentDetailDto;

// ── Timeline ─────────────────────────────────────────────────────────

export type ShipmentTimelineItemDto = {
	seq: number;
	event_type: string;
	scb: string;
	payload?: Record<string, unknown> | null;
	created_at?: string | null;
};

export type GetShipmentTimelineResponseDto = ShipmentTimelineItemDto[] | {
	timeline: ShipmentTimelineItemDto[];
};

// ── Create ───────────────────────────────────────────────────────────

export type CreateShipmentRequestDto = {
	client_id: string;
	current_office_id?: string | null;
	notes?: string | null;
};

export type CreateShipmentResponseDto = {
	shipment_id?: string;
	shipment?: { id: string };
};

// ── Change Status ────────────────────────────────────────────────────

/** Matches Rust ChangeStatusRequest: to_status is SCREAMING_SNAKE_CASE */
export type ChangeShipmentStatusRequestDto = {
	to_status: string;
	to_office_id?: string | null;
	notes?: string | null;
};
