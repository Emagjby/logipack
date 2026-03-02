import type {
	ClientDto,
	ClientListItemDto,
	ListClientsResponseDto,
	GetClientResponseDto,
	CreateClientResponseDto,
	UpdateClientResponseDto,
} from "../dto/clients";
import {
	requireIsoDateTime,
	requireRecord,
	requireString,
	cleanNullableString,
	HubApiMappingError,
} from "../normalizers";

/**
 * App-safe shapes used by UI
 */
export type ClientListItem = {
	id: string;
	name: string;
	email?: string | null;
	phone?: string | null;
	updated_at?: string;
};

export type ClientDetail = {
	id: string;
	name: string;
	email?: string | null;
	phone?: string | null;

	created_at?: string;
	updated_at?: string;
	deleted_at?: string | null;
};

export function mapClientBase(args: {
	endpoint: string;
	obj: Record<string, unknown>;
}): {
	id: string;
	name: string;
	email?: string | null;
	phone?: string | null;
} {
	const { endpoint, obj } = args;

	const email =
		obj.email === undefined
			? undefined
			: cleanNullableString({
				endpoint,
				field: "client.email",
				value: obj.email,
			});

	const phone =
		obj.phone === undefined
			? undefined
			: cleanNullableString({
				endpoint,
				field: "client.phone",
				value: obj.phone,
			});

	return {
		id: requireString({
			endpoint,
			field: "client.id",
			value: obj.id,
			nonEmpty: true,
		}),
		name: requireString({
			endpoint,
			field: "client.name",
			value: obj.name,
			nonEmpty: true,
		}),
		...(email !== undefined ? { email } : {}),
		...(phone !== undefined ? { phone } : {}),
	};
}

export function mapClientListItemDtoToClientListItem(
	dto: ClientListItemDto,
): ClientListItem {
	const endpoint = "GET /admin/clients";
	const obj = requireRecord({ endpoint, field: "client", value: dto });

	const base = mapClientBase({ endpoint, obj });

	const updated_at =
		obj.updated_at === undefined || obj.updated_at === null
			? undefined
			: requireIsoDateTime({
				endpoint,
				field: "client.updated_at",
				value: obj.updated_at,
			});

	return {
		...base,
		...(updated_at ? { updated_at } : {}),
	};
}

export function mapClientDtoToClientDetail(dto: ClientDto): ClientDetail {
	const endpoint = "GET /admin/clients/:id";
	const obj = requireRecord({ endpoint, field: "client", value: dto });

	const base = mapClientBase({ endpoint, obj });

	const created_at =
		obj.created_at === undefined || obj.created_at === null
			? undefined
			: requireIsoDateTime({
				endpoint,
				field: "client.created_at",
				value: obj.created_at,
			});

	const updated_at =
		obj.updated_at === undefined || obj.updated_at === null
			? undefined
			: requireIsoDateTime({
				endpoint,
				field: "client.updated_at",
				value: obj.updated_at,
			});

	const deleted_at =
		obj.deleted_at === undefined
			? undefined
			: cleanNullableString({
				endpoint,
				field: "client.deleted_at",
				value: obj.deleted_at,
			});

	return {
		...base,
		...(created_at ? { created_at } : {}),
		...(updated_at ? { updated_at } : {}),
		...(deleted_at !== undefined ? { deleted_at } : {}),
	};
}

export function mapListClientsResponseDto(
	dto: ListClientsResponseDto,
): ClientListItem[] {
	const endpoint = "GET /admin/clients";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.clients)) {
		throw new HubApiMappingError({
			endpoint,
			field: "clients",
			message: "expected clients[]",
		});
	}

	return obj.clients.map((c, i) =>
		mapClientListItemDtoToClientListItem(
			requireRecord({ endpoint, field: `clients[${i}]`, value: c }) as any,
		),
	);
}

export function mapGetClientResponseDto(
	dto: GetClientResponseDto,
): ClientDetail {
	const endpoint = "GET /admin/clients/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapClientDtoToClientDetail(
		requireRecord({
			endpoint,
			field: "client",
			value: obj.client,
		}) as any,
	);
}

export function mapCreateClientResponseDto(
	dto: CreateClientResponseDto,
): ClientDetail {
	const endpoint = "POST /admin/clients";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapClientDtoToClientDetail(
		requireRecord({
			endpoint,
			field: "client",
			value: obj.client,
		}) as any,
	);
}

export function mapUpdateClientResponseDto(
	dto: UpdateClientResponseDto,
): ClientDetail {
	const endpoint = "PUT /admin/clients/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapClientDtoToClientDetail(
		requireRecord({
			endpoint,
			field: "client",
			value: obj.client,
		}) as any,
	);
}
