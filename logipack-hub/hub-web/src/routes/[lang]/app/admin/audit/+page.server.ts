import type { PageServerLoad } from "./$types";

export type AuditEvent = {
	id: string;
	at: string; // ISO datetime
	actor: string;
	action: string;
	entity_type?:
		| "shipment"
		| "office"
		| "client"
		| "employee"
		| "role"
		| "user"
		| "system";
	entity_id?: string;
	entity_label?: string; // e.g. "Shipment #SHP-1024"
};

type AuditSeed = Omit<AuditEvent, "at"> & { minutesAgo: number };

type AdminAuditResult =
	| { state: "ok"; events: AuditEvent[] }
	| { state: "empty"; events: AuditEvent[] }
	| { state: "error"; events: AuditEvent[] };

const AUDIT_SEED: AuditSeed[] = [
	{
		id: "audit-001",
		minutesAgo: 3,
		actor: "Nikolay Georgiev",
		action: "Updated shipment status to In Transit",
		entity_type: "shipment",
		entity_id: "SHP-2104",
		entity_label: "Shipment #SHP-2104",
	},
	{
		id: "audit-002",
		minutesAgo: 8,
		actor: "Maria Petrova",
		action: "Created office profile",
		entity_type: "office",
		entity_id: "office-sofia-hq",
		entity_label: "Office Sofia HQ",
	},
	{
		id: "audit-003",
		minutesAgo: 14,
		actor: "Elena Stoyanova",
		action: "Assigned employee to office",
		entity_type: "employee",
		entity_id: "emp-102",
		entity_label: "Employee EMP-102",
	},
	{
		id: "audit-004",
		minutesAgo: 22,
		actor: "System",
		action: "Generated shipment reconciliation batch",
		entity_type: "system",
		entity_id: "recon-20260222-01",
		entity_label: "Daily reconciliation",
	},
	{
		id: "audit-005",
		minutesAgo: 37,
		actor: "Ivan Dimitrov",
		action: "Updated client billing preferences",
		entity_type: "client",
		entity_id: "client-acme",
		entity_label: "Client ACME",
	},
	{
		id: "audit-006",
		minutesAgo: 49,
		actor: "Nikolay Georgiev",
		action: "Created shipment",
		entity_type: "shipment",
		entity_id: "SHP-2103",
		entity_label: "Shipment #SHP-2103",
	},
	{
		id: "audit-007",
		minutesAgo: 64,
		actor: "System",
		action: "Applied role policy refresh",
		entity_type: "role",
		entity_id: "admin",
		entity_label: "Role admin",
	},
	{
		id: "audit-008",
		minutesAgo: 79,
		actor: "Maria Petrova",
		action: "Updated office routing notes",
		entity_type: "office",
		entity_id: "office-varna-port",
		entity_label: "Office Varna Port",
	},
	{
		id: "audit-009",
		minutesAgo: 96,
		actor: "Georgi Dimitrov",
		action: "Deactivated user session",
		entity_type: "user",
		entity_id: "user-309",
		entity_label: "User U-309",
	},
];

function listMockAuditEvents(): AuditEvent[] {
	const now = Date.now();
	return AUDIT_SEED.map((event) => ({
		...event,
		at: new Date(now - event.minutesAgo * 60_000).toISOString(),
	})).sort((a, b) => b.at.localeCompare(a.at));
}

async function fetchAdminAuditEvents(): Promise<AdminAuditResult> {
	// TODO: Replace this in-memory mock with hub-api audit events.
	const events = listMockAuditEvents();
	return events.length > 0
		? { state: "ok", events }
		: { state: "empty", events: [] };
}

export const load: PageServerLoad = async () => {
	try {
		const result = await fetchAdminAuditEvents();
		return { result };
	} catch {
		return {
			result: {
				state: "error" as const,
				events: [],
			},
		};
	}
};
