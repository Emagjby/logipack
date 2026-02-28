import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ locals, cookies }) => {
	const authenticated = locals.session ?? cookies.get("lp_session");

	if (authenticated) {
		redirect(302, "/app");
	}

	return {
		appName: "LogiPack",
		loginUrl: "/login",
		features: [
			{
				title: "Shipment timeline",
				description:
					"Chronological audit of every event — who changed what, when.",
				icon: "timeline",
			},
			{
				title: "Office-aware tracking",
				description:
					"See the current office, track handoffs between locations.",
				icon: "office",
			},
			{
				title: "Role-based consoles",
				description: "Separate capabilities for employees and administrators.",
				icon: "roles",
			},
			{
				title: "Admin management",
				description: "Manage clients, offices, employees, and assignments.",
				icon: "admin",
			},
		],
		steps: [
			{
				number: 1,
				title: "Create or ingest",
				description: "Add shipments into the system.",
			},
			{
				number: 2,
				title: "Move through statuses",
				description: "Track across offices and status changes.",
			},
			{
				number: 3,
				title: "Inspect the timeline",
				description: "Full accountability at every step.",
			},
		],
	};
};
