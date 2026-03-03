import type {
	EmployeeDto,
	EmployeeListItemDto,
	GetEmployeeResponseDto,
	ListEmployeesResponseDto,
	CreateEmployeeResponseDto,
	UpdateEmployeeResponseDto,
} from "../dto/employees";
import type { OfficeDto } from "../dto/offices";
import {
	cleanNullableString,
	HubApiMappingError,
	requireIsoDateTime,
	requireRecord,
	requireString,
} from "../normalizers";

export type EmployeeListItem = {
	id: string;
	user_id: string;
	user_display_name?: string | null;
	full_name: string;
	email: string;

	office_id?: string | null;
	office_name?: string | null;
	office_city?: string | null;
};

export type EmployeeDetail = {
	id: string;
	user_id: string;
	full_name: string;
	user_display_name?: string | null;
	email: string;

	office_id?: string | null;
	office_name?: string | null;
	office_city?: string | null;

	created_at?: string;
	updated_at?: string;
	deleted_at?: string | null;
};

function mapOfficesListToSingleOfficeDetails(
	dto: OfficeDto[] | null | undefined,
): { office_id: string; office_name: string; office_city: string } {
	if (dto === undefined || dto === null) {
		return {
			office_id: "",
			office_name: "",
			office_city: "",
		};
	}

	if (!Array.isArray(dto) || dto.length === 0) {
		return {
			office_id: "",
			office_name: "",
			office_city: "",
		};
	}

	const office = dto[0];

	const office_id = requireString({
		endpoint: "employee.offices",
		field: "office.id",
		value: office.id,
		nonEmpty: true,
	});
	const office_name = requireString({
		endpoint: "employee.offices",
		field: "office.name",
		value: office.name,
		nonEmpty: true,
	});
	const office_city = requireString({
		endpoint: "employee.offices",
		field: "office.city",
		value: office.city,
		nonEmpty: true,
	});

	return {
		office_id,
		office_name,
		office_city,
	};
}

export function mapEmployeeBase(args: {
	endpoint: string;
	obj: Record<string, unknown>;
}): {
	id: string;
	user_id: string;
	full_name: string;
	user_display_name?: string | null;
	email: string;
} {
	const { endpoint, obj } = args;

	const user_display_name =
		obj.user_display_name === undefined
			? undefined
			: cleanNullableString({
				endpoint,
				field: "employee.user_display_name",
				value: obj.user_display_name,
			});

	return {
		id: requireString({
			endpoint,
			field: "employee.id",
			value: obj.id,
			nonEmpty: true,
		}),
		user_id: requireString({
			endpoint,
			field: "employee.user_id",
			value: obj.user_id,
			nonEmpty: true,
		}),
		full_name: requireString({
			endpoint,
			field: "employee.full_name",
			value: obj.full_name,
			nonEmpty: true,
		}),
		email: requireString({
			endpoint,
			field: "employee.email",
			value: obj.email,
			nonEmpty: true,
		}),
		...(user_display_name !== undefined ? { user_display_name } : {}),
	};
}

export function mapEmployeeListItemDtoToEmployeeListItem(
	dto: EmployeeListItemDto,
): EmployeeListItem {
	const endpoint = "GET /admin/employees";
	const obj = requireRecord({ endpoint, field: "employee", value: dto });

	const base = mapEmployeeBase({ endpoint, obj });
	const { office_id, office_name, office_city } =
		mapOfficesListToSingleOfficeDetails(
			obj.offices as OfficeDto[] | null | undefined,
		);

	return {
		...base,
		...(office_id ? { office_id } : {}),
		...(office_name ? { office_name } : {}),
		...(office_city ? { office_city } : {}),
	};
}

export function mapEmployeeDtoToEmployeeDetail(
	dto: EmployeeDto,
): EmployeeDetail {
	const endpoint = "GET /admin/employees/:id";
	const obj = requireRecord({ endpoint, field: "employees", value: dto });

	const base = mapEmployeeBase({ endpoint, obj });
	const { office_id, office_name, office_city } =
		mapOfficesListToSingleOfficeDetails(
			obj.offices as OfficeDto[] | null | undefined,
		);

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
		...(office_id ? { office_id } : {}),
		...(office_name ? { office_name } : {}),
		...(office_city ? { office_city } : {}),
		...(created_at ? { created_at } : {}),
		...(updated_at ? { updated_at } : {}),
		...(deleted_at !== undefined ? { deleted_at } : {}),
	};
}

export function mapListEmployeesResponseDto(
	dto: ListEmployeesResponseDto,
): EmployeeListItem[] {
	const endpoint = "GET /admin/employees";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	if (!Array.isArray(obj.employees)) {
		throw new HubApiMappingError({
			endpoint,
			field: "employees",
			message: "expected employee[]",
		});
	}

	return obj.employees.map((c, i) =>
		mapEmployeeListItemDtoToEmployeeListItem(
			requireRecord({ endpoint, field: `employee[${i}]`, value: c }) as any,
		),
	);
}

export function mapGetEmployeeResponseDto(
	dto: GetEmployeeResponseDto,
): EmployeeDetail {
	const endpoint = "GET /admin/employees/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapEmployeeDtoToEmployeeDetail(
		requireRecord({
			endpoint,
			field: "employee",
			value: obj.employee,
		}) as any,
	);
}

export function mapCreateEmployeeResponseDto(
	dto: CreateEmployeeResponseDto,
): EmployeeDetail {
	const endpoint = "POST /admin/employees";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapEmployeeDtoToEmployeeDetail(
		requireRecord({
			endpoint,
			field: "employee",
			value: obj.employee,
		}) as any,
	);
}

export function mapUpdateEmployeeResponseDto(
	dto: UpdateEmployeeResponseDto,
): EmployeeDetail {
	const endpoint = "PUT /admin/employees/:id";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	return mapEmployeeDtoToEmployeeDetail(
		requireRecord({
			endpoint,
			field: "employee",
			value: obj.employee,
		}) as any,
	);
}
