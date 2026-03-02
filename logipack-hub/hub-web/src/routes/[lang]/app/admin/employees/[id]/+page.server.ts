// import {
// 	deleteMockEmployee,
// 	getMockEmployeeById,
// } from "$lib/server/mockEmployees";
// import { fail, redirect } from "@sveltejs/kit";
// import type { Actions, PageServerLoad } from "./$types";
//
// type EmployeeDetail = {
// 	id: string;
// 	user_id: string;
// 	full_name: string;
// 	user_display_name: string | null;
// 	office_ids: string[];
// };
//
// type UserDetail = {
// 	name: string;
// 	email: string;
// };
//
// type OfficeDetail = {
// 	id: string;
// 	name: string;
// 	city: string;
// 	address: string;
// };
//
// type DetailResult =
// 	| {
// 		state: "ok";
// 		employee: EmployeeDetail;
// 		user: UserDetail;
// 		office: OfficeDetail | null;
// 		hasMultipleOffices: boolean;
// 	}
// 	| { state: "not_found" }
// 	| { state: "error"; message: string };
//
// function normalizeOfficeIds(employee: {
// 	office_id: string | null;
// 	office_ids?: string[];
// }): string[] {
// 	if (Array.isArray(employee.office_ids) && employee.office_ids.length > 0) {
// 		return [
// 			...new Set(employee.office_ids.map((id) => id.trim()).filter(Boolean)),
// 		];
// 	}
//
// 	return employee.office_id ? [employee.office_id] : [];
// }
//
// function fetchEmployeeDetail(id: string): DetailResult {
// 	const employee = getMockEmployeeById(id);
// 	if (!employee) {
// 		return { state: "not_found" };
// 	}
//
// 	const officeIds = normalizeOfficeIds(employee);
// 	const primaryOfficeId = officeIds[0] ?? null;
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
// 		user: {
// 			name:
// 				employee.user_display_name ?? employee.full_name ?? employee.user_id,
// 			email: employee.email,
// 		},
// 		office: office
// 			? {
// 				id: office.id,
// 				name: office.name,
// 				city: office.city,
// 				address: office.address,
// 			}
// 			: null,
// 		hasMultipleOffices: officeIds.length > 1,
// 	};
// }
//
// export const load: PageServerLoad = async ({ params }) => {
// 	try {
// 		return { result: fetchEmployeeDetail(params.id) };
// 	} catch (error) {
// 		console.error("admin.employees.detail.load_failed", {
// 			employeeId: params.id,
// 			error,
// 		});
// 		return {
// 			result: {
// 				state: "error" as const,
// 				message: "admin.employees.detail.load_failed",
// 			},
// 		};
// 	}
// };
//
// export const actions: Actions = {
// 	delete: async ({ params }) => {
// 		try {
// 			const deleted = deleteMockEmployee(params.id);
// 			if (!deleted) {
// 				return fail(404, {
// 					submitError: "admin.employees.detail.not_found",
// 				});
// 			}
// 		} catch {
// 			return fail(500, {
// 				submitError: "admin.employees.detail.delete_failed",
// 			});
// 		}
//
// 		throw redirect(303, `/${params.lang ?? "en"}/app/admin/employees`);
// 	},
// };
