export type EnsureUserRequestDto = {
	email: string;
	name: string;
};

export type MeResponseDto = {
	role: string;
	office_ids?: string[];
	current_office_id?: string | null;
	current_office_name?: string | null;
	employee_id?: string | null;
};
