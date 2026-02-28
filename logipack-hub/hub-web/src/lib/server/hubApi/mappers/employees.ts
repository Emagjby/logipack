import type {
	EmployeeListItemDto,
	EmployeeDetailDto,
	EmployeeOfficeAssignmentDto,
} from "../dto/employees";

export type EmployeeListItem = Record<string, never>;
export type EmployeeDetail = Record<string, never>;
export type EmployeeOfficeAssignment = Record<string, never>;

export function mapEmployeeListItemDtoToEmployeeListItem(
	_dto: EmployeeListItemDto,
): EmployeeListItem {
	return {};
}

export function mapEmployeeDetailDtoToEmployeeDetail(
	_dto: EmployeeDetailDto,
): EmployeeDetail {
	return {};
}

export function mapEmployeeOfficeAssignmentDtoToEmployeeOfficeAssignment(
	_dto: EmployeeOfficeAssignmentDto,
): EmployeeOfficeAssignment {
	return {};
}
