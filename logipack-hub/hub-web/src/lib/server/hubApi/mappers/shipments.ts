import type { ShipmentListItemDto, ShipmentDetailDto } from "../dto/shipments";

export type ShipmentListItem = Record<string, never>;
export type ShipmentDetail = Record<string, never>;

export function mapShipmentListItemDtoToShipmentListItem(
	_dto: ShipmentListItemDto,
): ShipmentListItem {
	return {};
}

export function mapShipmentDetailDtoToShipmentDetail(
	_dto: ShipmentDetailDto,
): ShipmentDetail {
	return {};
}
