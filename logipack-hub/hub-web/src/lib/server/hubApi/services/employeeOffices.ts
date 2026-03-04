import type { HubApiClient } from "..";
import type { EmployeeOfficesResponseDto } from "../dto/employeeOffices";
import {
	mapEmployeeOfficesResponseDto,
	type OfficeAssignmentResult,
} from "../mappers/employeeOffices";

export async function listEmployeeOffices(
	client: HubApiClient,
	employeeId: string,
	timeoutMs = 10_000,
): Promise<OfficeAssignmentResult> {
	const res = await client.get<EmployeeOfficesResponseDto>(
		`/admin/employees/${employeeId}/offices`,
		{ timeoutMs },
	);
	return mapEmployeeOfficesResponseDto(res.data);
}

export async function assignEmployeeOffice(
	client: HubApiClient,
	employeeId: string,
	officeId: string,
	timeoutMs = 10_000,
): Promise<void> {
	await client.post(
		`/admin/employees/${employeeId}/offices`,
		{ office_id: officeId },
		{ timeoutMs },
	);
}

export async function removeEmployeeOffice(
	client: HubApiClient,
	employeeId: string,
	officeId: string,
	timeoutMs = 10_000,
): Promise<void> {
	await client.delete(`/admin/employees/${employeeId}/offices/${officeId}`, {
		timeoutMs,
	});
}
