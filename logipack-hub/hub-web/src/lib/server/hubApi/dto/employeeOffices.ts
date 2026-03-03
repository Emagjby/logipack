import type { EmployeeDto } from "./employees";
import type { OfficeDto } from "./offices";

export type EmployeeOfficesResponseDto = {
	employee_id: string;

	offices: OfficeDto[];

	office_ids: string[];

	employee?: EmployeeDto | null;
	assigned_offices?: OfficeDto[] | null;
};

export type AssignEmployeeOfficeRequestDto = {
	office_id: string;
};
