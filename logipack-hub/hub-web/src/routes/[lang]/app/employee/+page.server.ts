import type { PageServerLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { _ } from "svelte-i18n";
import { get } from "svelte/store";
import { resolveEmployeeOffice } from "$lib/server/employeeOffice";

function f(key: string, vars?: Record<string, any>): string {
	return get(_)(key, vars);
}

export const load: PageServerLoad = async ({ parent }) => {
	const { session } = await parent();

	const now = new Date();
	const office = resolveEmployeeOffice(session);
	const canCreateShipment = Boolean(office.id);

	return {
		canCreateShipment,
		today: now.toISOString(),
		lastRefresh: now.toISOString(),
		greeting: getGreeting(session?.name ?? "there"),
		kpis: [
			{
				label: f("empd.actives"),
				value: 12,
				change: `+3 ${f("this_week")}`,
				context: `vs 9 ${f("last_week")}`,
				trend: "up" as const,
				severity: "good" as const,
				sparkline: [4, 6, 5, 8, 7, 10, 12],
			},
			{
				label: f("empd.pendings"),
				value: 5,
				change: `2 ${f("urgent")}`,
				context: `${f("sla.target")}: < 3`,
				trend: "neutral" as const,
				severity: "warn" as const,
				sparkline: [2, 3, 4, 3, 5, 4, 5],
			},
			{
				label: f("empd.deliveries"),
				value: 8,
				change: `+2 vs ${f("yesterday")}`,
				context: `${f("daily.target")}: 5`,
				trend: "up" as const,
				severity: "good" as const,
				sparkline: [3, 5, 6, 4, 7, 6, 8],
			},
		],
		recentShipments: [
			{
				id: "SHP-1042",
				destination: "Sofia HQ",
				status: "in-transit" as const,
				eta: "Feb 20",
				priority: "high" as const,
			},
			{
				id: "SHP-1041",
				destination: "Varna Port",
				status: "delivered" as const,
				eta: "Feb 18",
				priority: "normal" as const,
			},
			{
				id: "SHP-1040",
				destination: "Plovdiv Office",
				status: "pending" as const,
				eta: "Feb 22",
				priority: "normal" as const,
			},
			{
				id: "SHP-1039",
				destination: "Veliko Tarnovo Office",
				status: "in-transit" as const,
				eta: "Feb 19",
				priority: "high" as const,
			},
			{
				id: "SHP-1038",
				destination: "Sofia HQ",
				status: "delivered" as const,
				eta: "Feb 17",
				priority: "normal" as const,
			},
		],
		recentSearches: ["SHP-1042", "SHP-1039", "SHP-1038"],
		activity: [
			{
				group: f("today"),
				items: [
					{
						time: "09:15",
						shipmentId: "SHP-1042",
						title: `${f("delivered")} - Sofia HQ`,
						tag: "Delivered" as const,
					},
					{
						time: "08:40",
						shipmentId: "SHP-1041",
						title: `${f("delivered")} - Plovdiv Office`,
						tag: "Delivered" as const,
					},
					{
						time: "08:10",
						shipmentId: "SHP-1043",
						title: `${f("awaits_pickup")} - Varna Port`,
						tag: "Pending" as const,
					},
				],
			},
			{
				group: f("yesterday"),
				items: [
					{
						time: "17:30",
						shipmentId: "SHP-1039",
						title: `${f("departed")} - Veliko Tarnovo Office`,
						tag: "In Transit" as const,
					},
					{
						time: "14:15",
						shipmentId: "SHP-1038",
						title: `${f("delivered")} - Sofia HQ`,
						tag: "Delivered" as const,
					},
					{
						time: "09:00",
						shipmentId: "SHP-1037",
						title: `${f("cancelled")} - Plovdiv Office`,
						tag: "Cancelled" as const,
					},
				],
			},
		],
	};
};

function getGreeting(name: string): string {
	const h = new Date().getHours();
	const firstName = name.split(" ")[0];
	if (h < 12) return `${f("greet.morning")}, ${firstName}`;
	if (h < 18) return `${f("greet.afternoon")}, ${firstName}`;
	return `${f("greet.evening")}, ${firstName}`;
}
