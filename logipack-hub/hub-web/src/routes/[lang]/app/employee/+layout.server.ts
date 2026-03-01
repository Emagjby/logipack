import type { LayoutServerLoad } from "./$types";
import { error } from "@sveltejs/kit";

export const load: LayoutServerLoad = async ({ parent }) => {
    const { session } = await parent();

    if (session?.role !== "employee") {
        throw error(403, "error.details.employee_only");
    }

    return {};
};
