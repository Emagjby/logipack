import type { EmployeeDetail, EmployeeListItem, HubApiClient } from "../index";
import type {
	CreateEmployeeRequestDto,
	CreateEmployeeResponseDto,
	ListEmployeesResponseDto,
	GetEmployeeResponseDto,
	UpdateEmployeeRequestDto,
	UpdateEmployeeResponseDto,
} from "../dto/employees";
import {
	mapCreateEmployeeResponseDto,
	mapListEmployeesResponseDto,
	mapGetEmployeeResponseDto,
	mapUpdateEmployeeResponseDto,
} from "../mappers/employees";

export async function listEmployees(
	client: HubApiClient,
	timeoutMs = 10_000,
): Promise<EmployeeListItem[]> {
	const res = await client.get<ListEmployeesResponseDto>("/admin/employees", {
		timeoutMs,
	});
	return mapListEmployeesResponseDto(res.data);
}

export async function getEmployee(
	client: HubApiClient,
	employeeId: string,
	timeoutMs = 10_000,
): Promise<EmployeeDetail> {
	const res = await client.get<GetEmployeeResponseDto>(
		`/admin/employees/${employeeId}`,
		{ timeoutMs },
	);
	return mapGetEmployeeResponseDto(res.data);
}

export async function createEmployee(
	client: HubApiClient,
	payload: CreateEmployeeRequestDto,
	timeoutMs = 10_000,
): Promise<EmployeeDetail> {
	const res = await client.post<CreateEmployeeResponseDto>(
		"/admin/employees",
		payload,
		{ timeoutMs },
	);

	return mapCreateEmployeeResponseDto(res.data);
}

export async function updateEmployee(
	client: HubApiClient,
	employeeId: string,
	payload: UpdateEmployeeRequestDto,
	timeoutMs = 10_000,
): Promise<EmployeeDetail> {
	const res = await client.put<UpdateEmployeeResponseDto>(
		`/admin/employees/${employeeId}`,
		payload,
		{ timeoutMs },
	);
	return mapUpdateEmployeeResponseDto(res.data);
}

export async function deleteEmployee(
	client: HubApiClient,
	employeeId: string,
	timeoutMs = 10_000,
): Promise<void> {
	await client.delete(`/admin/employees/${employeeId}`, { timeoutMs });
}
