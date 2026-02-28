import type { ClientListItemDto, ClientDetailDto } from "../dto/clients";

export type ClientListItem = Record<string, never>;
export type ClientDetail = Record<string, never>;

export function mapClientListItemDtoToClientListItem(
	_dto: ClientListItemDto,
): ClientListItem {
	return {};
}

export function mapClientDetailDtoToClientDetail(
	_dto: ClientDetailDto,
): ClientDetail {
	return {};
}
