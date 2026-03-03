import type { EmployeeOfficesResponseDto } from "../dto/employeeOffices";
import {
	cleanNullableString,
	HubApiMappingError,
	requireRecord,
	requireString,
} from "../normalizers";

export type EmployeeContext = {
	id: string;
	user_id: string;
	full_name: string;
	user_display_name: string | null;
	office_ids: string[];
};

export type OfficeContext = {
	id: string;
	name: string;
	city: string;
	address: string;
};

export type OfficeAssignmentResult = {
	state: "ok";
	employee: EmployeeContext;
	offices: OfficeContext[];
	currentOfficeId: string | null;
	currentOffice: OfficeContext | null;
	hasMultipleOffices: boolean;
};

function mapOfficeContext(
	endpoint: string,
	obj: Record<string, unknown>,
): OfficeContext {
	const id = requireString({
		endpoint,
		field: "office.id",
		value: obj.id,
		nonEmpty: true,
	});
	const name = requireString({
		endpoint,
		field: "office.name",
		value: obj.name,
		nonEmpty: true,
	});
	const city = requireString({
		endpoint,
		field: "office.city",
		value: obj.city,
		nonEmpty: true,
	});
	const address = requireString({
		endpoint,
		field: "office.address",
		value: obj.address,
		nonEmpty: true,
	});

	return {
		id,
		name,
		city,
		address,
	};
}

function normalizeOfficeIds(endpoint: string, ids: unknown): string[] {
	if (!Array.isArray(ids)) {
		throw new HubApiMappingError({
			endpoint,
			field: "office_ids",
			message: "expected office_ids[]",
		});
	}

	return ids
		.map((v, i) => {
			if (typeof v === "string") {
				return requireString({
					endpoint,
					field: `office_ids[${i}]`,
					value: v,
					nonEmpty: true,
				});
			}

			if (v && typeof v === "object") {
				const obj = requireRecord({
					endpoint,
					field: `office_ids[${i}]`,
					value: v,
				});
				return requireString({
					endpoint,
					field: `office_ids[${i}].id`,
					value: obj.id,
					nonEmpty: true,
				});
			}

			throw new HubApiMappingError({
				endpoint,
				field: `office_ids[${i}]`,
				message: "expected office id string",
			});
		})
		.map((s) => s.trim())
		.filter(Boolean);
}

function mapEmployeeContextFromHydratedEmployee(
	endpoint: string,
	e: Record<string, unknown>,
): EmployeeContext {
	const id = requireString({
		endpoint,
		field: "employee.id",
		value: e.id,
		nonEmpty: true,
	});
	const user_id = requireString({
		endpoint,
		field: "employee.user_id",
		value: e.user_id,
		nonEmpty: true,
	});

	let user_display_name: string | null = null;
	let email: string | null = null;

	if (e.user !== undefined && e.user !== null) {
		const u = requireRecord({
			endpoint,
			field: "employee.user",
			value: e.user,
		});
		user_display_name =
			u.name === undefined
				? null
				: cleanNullableString({
					endpoint,
					field: "user.name",
					value: u.name,
				});
		email = requireString({
			endpoint,
			field: "user.email",
			value: u.email,
			nonEmpty: true,
		});
	}

	const full_name = user_display_name ?? email ?? user_id;

	const office_ids: string[] = [];

	return {
		id,
		user_id,
		full_name,
		user_display_name,
		office_ids,
	};
}

export function mapEmployeeOfficesResponseDto(
	dto: EmployeeOfficesResponseDto,
): OfficeAssignmentResult {
	const endpoint = "GET /admin/employees/:id/offices";
	const obj = requireRecord({ endpoint, field: "response", value: dto });

	const employee_id = requireString({
		endpoint,
		field: "employee_id",
		value: obj.employee_id,
		nonEmpty: true,
	});

	if (!Array.isArray(obj.offices)) {
		throw new HubApiMappingError({
			endpoint,
			field: "offices",
			message: "expected offices[]",
		});
	}
	const offices: OfficeContext[] = obj.offices.map((o, i) =>
		mapOfficeContext(
			endpoint,
			requireRecord({ endpoint, field: `offices[${i}]`, value: o }),
		),
	);

	const office_ids = normalizeOfficeIds(endpoint, obj.office_ids);

	const currentOfficeId = office_ids[0] ?? null;
	const currentOffice = currentOfficeId
		? (offices.find((o) => o.id === currentOfficeId) ?? null)
		: null;

	let employee: EmployeeContext;

	if (obj.employee !== undefined && obj.employee !== null) {
		employee = mapEmployeeContextFromHydratedEmployee(
			endpoint,
			requireRecord({
				endpoint,
				field: "employee",
				value: obj.employee,
			}),
		);
		employee.office_ids = office_ids;
	} else {
		employee = {
			id: employee_id,
			user_id: "",
			full_name: employee_id,
			user_display_name: null,
			office_ids,
		};
	}

	return {
		state: "ok",
		employee,
		offices,
		currentOfficeId,
		currentOffice,
		hasMultipleOffices: office_ids.length > 1,
	};
}
