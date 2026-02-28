import type { OfficeListItemDto, OfficeDetailDto } from "../dto/offices";

export type OfficeListItem = Record<string, never>;
export type OfficeDetail = Record<string, never>;

export function mapOfficeListItemDtoToOfficeListItem(
	_dto: OfficeListItemDto,
): OfficeListItem {
	return {};
}

export function mapOfficeDetailDtoToOfficeDetail(
	_dto: OfficeDetailDto,
): OfficeDetail {
	return {};
}
