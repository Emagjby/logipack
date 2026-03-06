import type { HubApiClient } from "../index";
import type {
	ChangeShipmentStatusRequestDto,
	CreateShipmentRequestDto,
	CreateShipmentResponseDto,
	GetShipmentResponseDto,
	GetShipmentTimelineResponseDto,
	ListShipmentsResponseDto,
} from "../dto/shipments";
import type { ShipmentDetail, ShipmentListItem, ShipmentTimelineItem } from "../mappers/shipments";
import {
	buildChangeStatusRequestDto,
	mapCreateShipmentResponseDto,
	mapGetShipmentResponseDto,
	mapGetShipmentTimelineResponseDto,
	mapListShipmentsResponseDto,
} from "../mappers/shipments";

export async function listShipments(
	client: HubApiClient,
	timeoutMs = 10_000,
): Promise<ShipmentListItem[]> {
	const res = await client.get<ListShipmentsResponseDto>("/shipments", {
		timeoutMs,
	});
	return mapListShipmentsResponseDto(res.data);
}

export async function getShipment(
	client: HubApiClient,
	shipmentId: string,
	timeoutMs = 10_000,
): Promise<ShipmentDetail> {
	const res = await client.get<GetShipmentResponseDto>(
		`/shipments/${shipmentId}`,
		{ timeoutMs },
	);
	return mapGetShipmentResponseDto(res.data);
}

export async function getShipmentTimeline(
	client: HubApiClient,
	shipmentId: string,
	query?: URLSearchParams,
	timeoutMs = 10_000,
): Promise<ShipmentTimelineItem[]> {
	const suffix = query && query.size > 0 ? `?${query.toString()}` : "";
	const res = await client.get<GetShipmentTimelineResponseDto>(
		`/shipments/${shipmentId}/timeline${suffix}`,
		{ timeoutMs },
	);
	return mapGetShipmentTimelineResponseDto(res.data);
}

export async function createShipment(
	client: HubApiClient,
	payload: CreateShipmentRequestDto,
	timeoutMs = 10_000,
): Promise<{ id: string }> {
	const res = await client.post<CreateShipmentResponseDto>(
		"/shipments",
		payload,
		{ timeoutMs },
	);
	return mapCreateShipmentResponseDto(res.data);
}

/**
 * Change shipment status.
 * POST /shipments/:id/status → 204 No Content
 */
export async function changeShipmentStatus(
	client: HubApiClient,
	shipmentId: string,
	input: { to_status: string; to_office_id?: string | null; notes?: string | null },
	timeoutMs = 10_000,
): Promise<void> {
	const dto = buildChangeStatusRequestDto(input);
	await client.post<void>(
		`/shipments/${shipmentId}/status`,
		dto,
		{ timeoutMs },
	);
}
