import type { HubApiClient } from "../index";
import type {
	CreateOfficeRequestDto,
	CreateOfficeResponseDto,
	GetOfficeResponseDto,
	ListOfficesResponseDto,
	UpdateOfficeRequestDto,
	UpdateOfficeResponseDto,
} from "../dto/offices";
import type { OfficeDetail, OfficeListItem } from "../mappers/offices";
import {
	mapCreateOfficeResponseDto,
	mapGetOfficeResponseDto,
	mapListOfficesResponseDto,
	mapUpdateOfficeResponseDto,
} from "../mappers/offices";

export async function listOffices(
	client: HubApiClient,
	timeoutMs = 10_000,
): Promise<OfficeListItem[]> {
	const res = await client.get<ListOfficesResponseDto>("/admin/offices", {
		timeoutMs,
	});
	return mapListOfficesResponseDto(res.data);
}

export async function getOffice(
	client: HubApiClient,
	officeId: string,
	timeoutMs = 10_000,
): Promise<OfficeDetail> {
	const res = await client.get<GetOfficeResponseDto>(
		`/admin/offices/${officeId}`,
		{ timeoutMs },
	);
	return mapGetOfficeResponseDto(res.data);
}

export async function createOffice(
	client: HubApiClient,
	payload: CreateOfficeRequestDto,
	timeoutMs = 10_000,
): Promise<OfficeDetail> {
	const res = await client.post<CreateOfficeResponseDto>(
		"/admin/offices",
		payload,
		{ timeoutMs },
	);
	return mapCreateOfficeResponseDto(res.data);
}

export async function updateOffice(
	client: HubApiClient,
	officeId: string,
	payload: UpdateOfficeRequestDto,
	timeoutMs = 10_000,
): Promise<OfficeDetail> {
	const res = await client.put<UpdateOfficeResponseDto>(
		`/admin/offices/${officeId}`,
		payload,
		{ timeoutMs },
	);
	return mapUpdateOfficeResponseDto(res.data);
}

export async function deleteOffice(
	client: HubApiClient,
	officeId: string,
	timeoutMs = 10_000,
): Promise<void> {
	await client.delete(`/admin/offices/${officeId}`, { timeoutMs });
}
