export type ClientDto = {
	id: string;
	name: string;

	email?: string | null;
	phone?: string | null;

	createdAt?: string | null;
	updatedAt?: string | null;
	deletedAt?: string | null;
};

export type ClientListItemDto = {
	id: string;
	name: string;
	email?: string | null;
	phone?: string | null;

	updated_at?: string | null;
};

export type ListClientsResponseDto = {
	clients: ClientListItemDto[];
};

export type GetClientResponseDto = {
	client: ClientDto;
};

export type CreateClientRequestDto = {
	name: string;
	email?: string | null;
	phone?: string | null;
};

export type CreateClientResponseDto = {
	client: ClientDto;
};

export type UpdateClientRequestDto = {
	name?: string | null;
	email?: string | null;
	phone?: string | null;
};

export type UpdateClientResponseDto = {
	client: ClientDto;
};
