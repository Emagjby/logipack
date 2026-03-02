import type {
	OfficeDto,
	OfficeListItemDto,
	ListOfficesResponseDto,
	GetOfficeResponseDto,
	CreateOfficeResponseDto,
	UpdateOfficeResponseDto,
} from "../dto/offices";
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
export type OfficeListItem = {
	id: string;
	name: string;
	city: string;
	address: string;
	updated_at?: string;
};

export type OfficeDetail = {
	id: string;
	name: string;
	city: string;
	address: string;

	created_at?: string;
	updated_at?: string;
	deleted_at?: string | null;
};

export function mapOfficeListItemDtoToOfficeListItem(
	dto: OfficeListItemDto,
): OfficeListItem {
	const endpoint = "GET /admin/offices";
	const obj = requireRecord({ endpoint, field: "office", value: dto });

	return {
		id: requireString({
			endpoint,
			field: "office.id",
			value: obj.id,
			nonEmpty: true,
		}),
		name: requireString({
			endpoint,
			field: "office.name",
			value: obj.name,
			nonEmpty: true,
		}),
		city: requireString({
			endpoint,
			field: "office.city",
			value: obj.city,
			nonEmpty: true,
		}),
		address: requireString({
			endpoint,
			field: "office.address",
			value: obj.address,
			nonEmpty: true,
		}),
		...(obj.updated_at
			? {
				updated_at: requireIsoDateTime({
					endpoint,
					field: "office.updated_at",
					value: obj.updated_at,
				}),
			}
			: {}),
	};
}

export function mapOfficeDtoToOfficeDetail(dto: OfficeDto): OfficeDetail {
	const endpoint = "GET /admin/offices/:id";
	const obj = requireRecord({ endpoint, field: "office", value: dto });

	const created_at =
		obj.created_at === undefined
			? undefined
			: requireIsoDateTime({
				endpoint,
				field: "office.created_at",
				value: obj.created_at,
			});

	const deleted_at =
		obj.deleted_at === undefined
			? undefined
			: cleanNullableString({
				endpoint,
				field: "office.deleted_at",
				value: obj.deleted_at,
			});

	return {
		id: requireString({
			endpoint,
			field: "office.id",
			value: obj.id,
			nonEmpty: true,
		}),
		name: requireString({
			endpoint,
			field: "office.name",
			value: obj.name,
			nonEmpty: true,
		}),
		city: requireString({
			endpoint,
			field: "office.city",
			value: obj.city,
			nonEmpty: true,
		}),
		address: requireString({
			endpoint,
			field: "office.address",
			value: obj.address,
			nonEmpty: true,
		}),
		...(obj.updated_at
			? {
				updated_at: requireIsoDateTime({
					endpoint,
					field: "office.updated_at",
					value: obj.updated_at,
				}),
			}
			: {}),
		...(created_at !== undefined ? { created_at } : {}),
		...(deleted_at !== undefined ? { deleted_at } : {}),
	};
}

export function mapListOfficesResponseDto(
	dto: ListOfficesResponseDto,
): OfficeListItem[] {
	const endpoint = "GET /admin/offices";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	const offices = obj.offices;
	if (!Array.isArray(offices)) {
		throw new HubApiMappingError({
			endpoint,
			field: "offices",
			message: "expected offices[]",
		});
	}

	return offices.map((o, i) => {
		return mapOfficeListItemDtoToOfficeListItem(
			requireRecord({
				endpoint,
				field: `offices[${i}]`,
				value: o,
			}) as any,
		);
	});
}

export function mapGetOfficeResponseDto(
	dto: GetOfficeResponseDto,
): OfficeDetail {
	const endpoint = "GET /admin/offices/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapOfficeDtoToOfficeDetail(
		requireRecord({
			endpoint,
			field: "office",
			value: obj.office,
		}) as any,
	);
}

export function mapCreateOfficeResponseDto(
	dto: CreateOfficeResponseDto,
): OfficeDetail {
	const endpoint = "POST /admin/offices";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapOfficeDtoToOfficeDetail(
		requireRecord({
			endpoint,
			field: "office",
			value: obj.office,
		}) as any,
	);
}

export function mapUpdateOfficeResponseDto(
	dto: UpdateOfficeResponseDto,
): OfficeDetail {
	const endpoint = "PUT /admin/offices/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapOfficeDtoToOfficeDetail(
		requireRecord({
			endpoint,
			field: "office",
			value: obj.office,
		}) as any,
	);
}
