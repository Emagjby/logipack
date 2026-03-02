// import type { PageServerLoad } from "./$types";
// import { listMockEmployees } from "$lib/server/mockEmployees";
//
// type EmployeeListRow = {
// 	id: string;
// 	user_id: string;
// 	user_display_name: string | null;
// 	full_name: string;
// 	email: string;
// 	office_id: string | null;
// 	office_name: string | null;
// 	office_city: string | null;
// };
//
// type AdminEmployeesResult =
// 	| { state: "ok"; employees: EmployeeListRow[] }
// 	| { state: "empty"; employees: [] }
// 	| { state: "error"; employees: []; message?: string };
//
// async function fetchAdminEmployees(): Promise<AdminEmployeesResult> {
// 	const officesById = new Map(
// 		listMockOffices().map((office) => [office.id, office]),
// 	);
//
// 	const employees = listMockEmployees().map((employee) => {
// 		const office = employee.office_id
// 			? officesById.get(employee.office_id)
// 			: null;
// 		return {
// 			...employee,
// 			office_name: office?.name ?? null,
// 			office_city: office?.city ?? null,
// 		};
// 	});
//
// 	return employees.length > 0
// 		? { state: "ok", employees }
// 		: { state: "empty", employees: [] };
// }
//
// export const load: PageServerLoad = async () => {
// 	try {
// 		const result = await fetchAdminEmployees();
// 		return { result };
// 	} catch {
// 		return {
// 			result: {
// 				state: "error" as const,
// 				employees: [] as [],
// 				message: "admin.employees.error.generic",
// 			},
// 		};
// 	}
// };
