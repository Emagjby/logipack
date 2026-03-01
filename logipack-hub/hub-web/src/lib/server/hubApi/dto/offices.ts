export type OfficeDto = {
	id: string;
	name: string;
	city: string;
	address: string;

	created_at?: string | null;
	updated_at?: string | null;
	deleted_at?: string | null;
};

export type OfficeListItemDto = {
	id: string;
	name: string;
	city: string;
	address: string;

	updated_at?: string | null;
};

export type ListOfficesResponseDto = {
	offices: OfficeListItemDto[];
};

export type GetOfficeResponseDto = {
	office: OfficeDto;
};

export type CreateOfficeRequestDto = {
	name: string;
	city: string;
	address: string;
};

export type CreateOfficeResponseDto = {
	office: OfficeDto;
};

export type UpdateOfficeRequestDto = {
	name?: string | null;
	city?: string | null;
	address?: string | null;
};

export type UpdateOfficeResponseDto = {
	office: OfficeDto;
};
