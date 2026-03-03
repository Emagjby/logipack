import type { UserDto } from "./common";
import type { OfficeDto } from "./offices";

export type EmployeeDto = {
	id: string;
	user_id: string;
	full_name: string;
	user_display_name?: string | null;
	email: string;

	user?: UserDto;
	offices?: OfficeDto[] | null;

	created_at?: string | null;
	updated_at?: string | null;
	deleted_at?: string | null;
};

export type EmployeeListItemDto = {
	id: string;
	user_id: string;
	full_name: string;
	user_display_name?: string | null;
	email: string;
	offices?: OfficeDto[] | null;
};

export type ListEmployeesResponseDto = {
	employees: EmployeeListItemDto[];
};

export type GetEmployeeResponseDto = {
	employee: EmployeeDto;
};

export type CreateEmployeeRequestDto = {
	email: string;
};

export type CreateEmployeeResponseDto = {
	employee: EmployeeDto;
};

export type UpdateEmployeeRequestDto = {
	email?: string | null;
};

export type UpdateEmployeeResponseDto = {
	employee: EmployeeDto;
};
