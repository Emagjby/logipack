export type AuditEventDto = {
	id: string;
	occurred_at: string;
	actor_user_id?: string | null;
	actor_display_name?: string | null;
	action_key: string;
	entity_type?: string | null;
	entity_id?: string | null;
	entity_label?: string | null;
	office_id?: string | null;
	office_label?: string | null;
	target_route?: string | null;
	metadata?: Record<string, unknown> | null;
};

export type AuditPageDto = {
	limit: number;
	next_cursor?: string | null;
	has_next: boolean;
};

export type ListAuditResponseDto = {
	events: AuditEventDto[];
	page: AuditPageDto;
};
