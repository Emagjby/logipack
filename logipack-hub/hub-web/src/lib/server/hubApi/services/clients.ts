import type { HubApiClient } from "../index";
import type {
	CreateClientRequestDto,
	CreateClientResponseDto,
	GetClientResponseDto,
	ListClientsResponseDto,
	UpdateClientRequestDto,
	UpdateClientResponseDto,
} from "../dto/clients";
import type { ClientDetail, ClientListItem } from "../mappers/clients";
import {
	mapCreateClientResponseDto,
	mapGetClientResponseDto,
	mapListClientsResponseDto,
	mapUpdateClientResponseDto,
} from "../mappers/clients";

export async function listClients(
	client: HubApiClient,
	timeoutMs = 10_000,
): Promise<ClientListItem[]> {
	const res = await client.get<ListClientsResponseDto>("/admin/clients", {
		timeoutMs,
	});
	return mapListClientsResponseDto(res.data);
}

export async function getClient(
	client: HubApiClient,
	clientId: string,
	timeoutMs = 10_000,
): Promise<ClientDetail> {
	const res = await client.get<GetClientResponseDto>(
		`/admin/clients/${clientId}`,
		{ timeoutMs },
	);
	return mapGetClientResponseDto(res.data);
}

export async function createClient(
	client: HubApiClient,
	payload: CreateClientRequestDto,
	timeoutMs = 10_000,
): Promise<ClientDetail> {
	const res = await client.post<CreateClientResponseDto>(
		"/admin/clients",
		payload,
		{ timeoutMs },
	);
	return mapCreateClientResponseDto(res.data);
}

export async function updateClient(
	client: HubApiClient,
	clientId: string,
	payload: UpdateClientRequestDto,
	timeoutMs = 10_000,
): Promise<ClientDetail> {
	const res = await client.put<UpdateClientResponseDto>(
		`/admin/clients/${clientId}`,
		payload,
		{ timeoutMs },
	);
	return mapUpdateClientResponseDto(res.data);
}

export async function deleteClient(
	client: HubApiClient,
	clientId: string,
	timeoutMs = 10_000,
): Promise<void> {
	await client.delete(`/admin/clients/${clientId}`, { timeoutMs });
}
