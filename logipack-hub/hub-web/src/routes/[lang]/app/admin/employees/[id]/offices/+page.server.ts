// import {
// 	assignMockEmployeeOffice,
// 	getMockEmployeeById,
// } from "$lib/server/mockEmployees";
// import { getMockOfficeById, listMockOffices } from "$lib/server/mockOffices";
// import { fail, redirect } from "@sveltejs/kit";
// import type { Actions, PageServerLoad } from "./$types";
//
// type EmployeeContext = {
// 	id: string;
// 	user_id: string;
// 	full_name: string;
// 	user_display_name: string | null;
// 	office_ids: string[];
// };
//
// type OfficeContext = {
// 	id: string;
// 	name: string;
// 	city: string;
// 	address: string;
// };
//
// type OfficeAssignmentResult =
// 	| {
// 			state: "ok";
// 			employee: EmployeeContext;
// 			offices: OfficeContext[];
// 			currentOfficeId: string | null;
// 			currentOffice: OfficeContext | null;
// 			hasMultipleOffices: boolean;
// 	  }
// 	| { state: "not_found" }
// 	| { state: "error"; message: string };
//
// type AssignOfficeValues = {
// 	office_id: string;
// };
//
// type AssignOfficeFieldErrors = {
// 	office_id?: string;
// };
//
// function normalizeOfficeIds(employee: {
// 	office_id: string | null;
// 	office_ids?: string[];
// }): string[] {
// 	if (Array.isArray(employee.office_ids) && employee.office_ids.length > 0) {
// 		return [...new Set(employee.office_ids.map((id) => id.trim()).filter(Boolean))];
// 	}
//
// 	return employee.office_id ? [employee.office_id] : [];
// }
//
// function toOfficeContext(office: {
// 	id: string;
// 	name: string;
// 	city: string;
// 	address: string;
// }): OfficeContext {
// 	return {
// 		id: office.id,
// 		name: office.name,
// 		city: office.city,
// 		address: office.address,
// 	};
// }
//
// function fetchOfficeAssignmentContext(id: string): OfficeAssignmentResult {
// 	const employee = getMockEmployeeById(id);
// 	if (!employee) {
// 		return { state: "not_found" };
// 	}
//
// 	const officeIds = normalizeOfficeIds(employee);
// 	const currentOfficeId = officeIds[0] ?? null;
// 	const offices = listMockOffices().map((office) => toOfficeContext(office));
// 	const currentOfficeRecord = currentOfficeId
// 		? getMockOfficeById(currentOfficeId)
// 		: null;
//
// 	return {
// 		state: "ok",
// 		employee: {
// 			id: employee.id,
// 			user_id: employee.user_id,
// 			full_name: employee.full_name,
// 			user_display_name: employee.user_display_name,
// 			office_ids: officeIds,
// 		},
// 		offices,
// 		currentOfficeId,
// 		currentOffice: currentOfficeRecord
// 			? toOfficeContext(currentOfficeRecord)
// 			: null,
// 		hasMultipleOffices: officeIds.length > 1,
// 	};
// }
//
// function parseAssignOfficeFormData(formData: FormData): AssignOfficeValues {
// 	return {
// 		office_id: String(formData.get("office_id") ?? "").trim(),
// 	};
// }
//
// function validateAssignOfficeForm(
// 	values: AssignOfficeValues,
// ): AssignOfficeFieldErrors {
// 	const fieldErrors: AssignOfficeFieldErrors = {};
//
// 	if (!values.office_id) {
// 		fieldErrors.office_id = "admin.employees.offices.form.office_required";
// 	}
//
// 	return fieldErrors;
// }
//
// function hasAssignOfficeFormErrors(
// 	fieldErrors: AssignOfficeFieldErrors,
// ): boolean {
// 	return Object.values(fieldErrors).some((value) => Boolean(value));
// }
//
// export const load: PageServerLoad = async ({ params }) => {
// 	try {
// 		return {
// 			result: fetchOfficeAssignmentContext(params.id),
// 		};
// 	} catch (error) {
// 		return {
// 			result: {
// 				state: "error" as const,
// 				message:
// 					error instanceof Error
// 						? error.message
// 						: "Unable to load office assignment right now.",
// 			},
// 		};
// 	}
// };
//
// export const actions: Actions = {
// 	assign: async ({ request, params }) => {
// 		const values = parseAssignOfficeFormData(await request.formData());
// 		const fieldErrors = validateAssignOfficeForm(values);
//
// 		if (hasAssignOfficeFormErrors(fieldErrors)) {
// 			return fail(400, { fieldErrors, submitError: null, values });
// 		}
//
// 		if (!getMockOfficeById(values.office_id)) {
// 			return fail(400, {
// 				fieldErrors: {
// 					office_id: "admin.employees.offices.form.office_invalid",
// 				} as AssignOfficeFieldErrors,
// 				submitError: null,
// 				values,
// 			});
// 		}
//
// 		if (!getMockEmployeeById(params.id)) {
// 			return fail(404, {
// 				fieldErrors: {} as AssignOfficeFieldErrors,
// 				submitError: "admin.employees.detail.not_found",
// 				values,
// 			});
// 		}
//
// 		try {
// 			const assignedEmployee = assignMockEmployeeOffice(params.id, values.office_id);
// 			if (!assignedEmployee) {
// 				return fail(500, {
// 					fieldErrors: {} as AssignOfficeFieldErrors,
// 					submitError: "admin.employees.offices.submit_failed",
// 					values,
// 				});
// 			}
// 		} catch {
// 			return fail(500, {
// 				fieldErrors: {} as AssignOfficeFieldErrors,
// 				submitError: "admin.employees.offices.submit_failed",
// 				values,
// 			});
// 		}
//
// 		throw redirect(
// 			303,
// 			`/${params.lang ?? "en"}/app/admin/employees/${params.id}/offices`,
// 		);
// 	},
// };
