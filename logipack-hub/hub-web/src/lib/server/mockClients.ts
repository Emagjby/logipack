export type MockClient = {
	id: string;
	name: string;
	email: string | null;
	phone: string | null;
	created_at: string;
	updated_at: string;
};

export type ClientInput = {
	name: string;
	email?: string | null;
	phone?: string | null;
};

export type CreateMockClientInput = ClientInput;
export type UpdateMockClientInput = ClientInput;

const SEED_CLIENTS: MockClient[] = [
	{
		id: "550e8400-e29b-41d4-a716-446655440000",
		name: "ACME Corporation",
		email: "contact@acme.com",
		phone: "+1 555 120 4410",
		created_at: "2026-01-08T09:20:00.000Z",
		updated_at: "2026-02-02T10:45:00.000Z",
	},
	{
		id: "6f14d4e2-9b40-4f01-91e5-8d34d5b6c1a2",
		name: "TechParts Distribution",
		email: "ops@techparts.dev",
		phone: "+1 555 884 2201",
		created_at: "2026-01-10T11:05:00.000Z",
		updated_at: "2026-02-04T12:00:00.000Z",
	},
	{
		id: "8c9a1b72-5df8-4e23-9e2d-1a5b7c3d9f10",
		name: "Greenline Logistics",
		email: null,
		phone: "+1 555 661 0090",
		created_at: "2026-01-14T08:50:00.000Z",
		updated_at: "2026-02-07T15:20:00.000Z",
	},
	{
		id: "f2b74c19-0d8a-4cfe-b721-3f8d2a9b6c44",
		name: "Nova Retail Group",
		email: "support@nova-retail.example",
		phone: null,
		created_at: "2026-01-16T13:40:00.000Z",
		updated_at: "2026-02-09T09:30:00.000Z",
	},
];

// Process-wide in-memory mock cache; resets on server restart.
const createdClients = new Map<string, MockClient>();
const updatedClients = new Map<string, MockClient>();
const deletedClientIds = new Set<string>();

function normalizeNullable(value: string | null | undefined): string | null {
	if (!value) return null;
	const trimmed = value.trim();
	return trimmed ? trimmed : null;
}

function listSeedClients(): MockClient[] {
	return SEED_CLIENTS.filter((client) => !deletedClientIds.has(client.id)).map(
		(client) => ({ ...client }),
	);
}

export function listMockClients(): MockClient[] {
	const clientsById = new Map(
		listSeedClients().map((client) => [client.id, client]),
	);

	for (const client of updatedClients.values()) {
		if (deletedClientIds.has(client.id)) continue;
		clientsById.set(client.id, { ...client });
	}

	for (const client of createdClients.values()) {
		if (deletedClientIds.has(client.id)) continue;
		clientsById.set(client.id, { ...client });
	}

	return [...clientsById.values()].sort((a, b) => a.name.localeCompare(b.name));
}

export function getMockClientById(id: string): MockClient | null {
	if (deletedClientIds.has(id)) return null;

	const createdClient = createdClients.get(id);
	if (createdClient) {
		return { ...createdClient };
	}

	const updatedClient = updatedClients.get(id);
	if (updatedClient) {
		return { ...updatedClient };
	}

	const seedClient = SEED_CLIENTS.find((client) => client.id === id);
	return seedClient ? { ...seedClient } : null;
}

export function filterMockClientsByQuery(
	clients: MockClient[],
	q: string,
): MockClient[] {
	const needle = q.trim().toLowerCase();
	if (!needle) return clients;

	return clients.filter((client) =>
		`${client.id} ${client.name} ${client.email ?? ""} ${client.phone ?? ""}`
			.toLowerCase()
			.includes(needle),
	);
}

export function createMockClient(input: ClientInput): { id: string } {
	const now = new Date().toISOString();
	const id =
		typeof crypto !== "undefined" && "randomUUID" in crypto
			? crypto.randomUUID()
			: `client_${Date.now()}`;
	deletedClientIds.delete(id);

	createdClients.set(id, {
		id,
		name: input.name.trim(),
		email: normalizeNullable(input.email),
		phone: normalizeNullable(input.phone),
		created_at: now,
		updated_at: now,
	});

	return { id };
}

export function updateMockClient(
	id: string,
	input: ClientInput,
): MockClient | null {
	const existing = getMockClientById(id);
	if (!existing) return null;

	const updatedClient: MockClient = {
		...existing,
		name: input.name.trim(),
		email: normalizeNullable(input.email),
		phone: normalizeNullable(input.phone),
		updated_at: new Date().toISOString(),
	};

	if (createdClients.has(id)) {
		createdClients.set(id, updatedClient);
		return { ...updatedClient };
	}

	updatedClients.set(id, updatedClient);
	return { ...updatedClient };
}

export function deleteMockClient(id: string): boolean {
	if (createdClients.has(id)) {
		createdClients.delete(id);
		return true;
	}

	if (updatedClients.has(id)) {
		updatedClients.delete(id);
		deletedClientIds.add(id);
		return true;
	}

	const existsInSeed = SEED_CLIENTS.some((client) => client.id === id);
	if (!existsInSeed) return false;

	deletedClientIds.add(id);
	return true;
}
