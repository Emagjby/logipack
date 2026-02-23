import { createHash, randomUUID } from "node:crypto";
import { listShipments } from "$lib/server/mockShipments";

export type MockOffice = {
	id: string;
	name: string;
	city: string;
	address: string;
	updated_at: string;
};

export type CreateMockOfficeInput = {
	name: string;
	city: string;
	address: string;
};

export type UpdateMockOfficeInput = {
	name: string;
	city: string;
	address: string;
};

const OFFICE_META: Record<string, { city: string; address: string }> = {
	"Sofia HQ": { city: "Sofia", address: "12 Vitosha Blvd" },
	"Plovdiv DC": {
		city: "Plovdiv",
		address: "45 Tsar Boris III Obedinitel Blvd",
	},
	"Varna Port": { city: "Varna", address: "8 Primorski Blvd" },
	"Burgas Hub": { city: "Burgas", address: "19 Transportna St" },
};

const OFFICE_UUIDS: Record<string, string> = {
	"Sofia HQ": "c2f4c5c1-2d59-4f05-8e0c-c3ef5f0c1fd3",
	"Plovdiv DC": "74f26d91-9a2a-4a2d-894e-9e3cf58ea6c7",
	"Varna Port": "15b3a4f0-50f4-4de8-b9e3-35d88c2c1d46",
	"Burgas Hub": "9f9a45f8-e2ce-4699-a00f-889b4d6dd1ca",
};

const NAMESPACE_UUID = "d6ddf7c1-8ef6-4a47-bf8d-0fdb77d0a7f6";

// createdOffices and updatedOffices are ephemeral in-memory mock caches.
// They are cleared on process restarts/cold starts and are not durable storage.
const createdOffices = new Map<string, MockOffice>();
const updatedOffices = new Map<string, MockOffice>();
const deletedOfficeIds = new Set<string>();

function uuidToBytes(uuid: string): Uint8Array {
	const normalized = uuid.replace(/-/g, "").toLowerCase();
	if (!/^[0-9a-f]{32}$/.test(normalized)) {
		throw new Error("invalid_uuid");
	}

	const bytes = new Uint8Array(16);
	for (let i = 0; i < 16; i++) {
		bytes[i] = Number.parseInt(normalized.slice(i * 2, i * 2 + 2), 16);
	}

	return bytes;
}

function bytesToUuid(bytes: Uint8Array): string {
	const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
	return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

function uuidV5(name: string, namespace: string): string {
	const nsBytes = uuidToBytes(namespace);
	const nameBytes = new TextEncoder().encode(name);
	const hash = createHash("sha1").update(nsBytes).update(nameBytes).digest();
	const bytes = new Uint8Array(hash.subarray(0, 16));

	// RFC4122 variant + version 5 bits.
	bytes[6] = (bytes[6] & 0x0f) | 0x50;
	bytes[8] = (bytes[8] & 0x3f) | 0x80;

	return bytesToUuid(bytes);
}

function toOfficeId(name: string): string {
	const knownUuid = OFFICE_UUIDS[name];
	if (knownUuid) return knownUuid;

	try {
		return uuidV5(name, NAMESPACE_UUID);
	} catch {
		return randomUUID();
	}
}

function buildBaseOfficesMap(): Map<string, MockOffice> {
	const officesById = new Map<string, MockOffice>();
	const latestByOffice = new Map<string, string>();

	for (const shipment of listShipments()) {
		const officeName = shipment.currentOfficeId?.trim();
		if (!officeName) continue;

		const prev = latestByOffice.get(officeName);
		if (!prev || shipment.updatedAt > prev) {
			latestByOffice.set(officeName, shipment.updatedAt);
		}
	}

	for (const office of [...latestByOffice.entries()]
		.map(([name, updated_at]) => {
			const meta = OFFICE_META[name] ?? {
				city: "Unknown",
				address: `${name} Address`,
			};
			return {
				id: toOfficeId(name),
				name,
				city: meta.city,
				address: meta.address,
				updated_at,
			};
		})) {
		if (deletedOfficeIds.has(office.id)) continue;
		officesById.set(office.id, office);
	}

	for (const office of createdOffices.values()) {
		if (deletedOfficeIds.has(office.id)) continue;
		officesById.set(office.id, office);
	}

	return officesById;
}

function getMockOfficesMap(): Map<string, MockOffice> {
	const officesById = buildBaseOfficesMap();
	for (const office of updatedOffices.values()) {
		if (deletedOfficeIds.has(office.id)) continue;
		officesById.set(office.id, office);
	}
	return officesById;
}

export function listMockOffices(): MockOffice[] {
	const officesById = getMockOfficesMap();
	return [...officesById.values()].sort((a, b) => a.name.localeCompare(b.name));
}

export function getMockOfficeById(id: string): MockOffice | null {
	if (deletedOfficeIds.has(id)) return null;
	return getMockOfficesMap().get(id) ?? null;
}

export function filterMockOfficesByQuery(
	offices: MockOffice[],
	q: string,
): MockOffice[] {
	const needle = q.trim().toLowerCase();
	if (!needle) return offices;

	return offices.filter((office) =>
		`${office.name} ${office.city} ${office.address}`
			.toLowerCase()
			.includes(needle),
	);
}

export function createMockOffice(input: CreateMockOfficeInput): { id: string } {
	const now = new Date().toISOString();
	const id =
		typeof crypto !== "undefined" && "randomUUID" in crypto
			? crypto.randomUUID()
			: `office_${Date.now()}`;
	deletedOfficeIds.delete(id);

	createdOffices.set(id, {
		id,
		name: input.name.trim(),
		city: input.city.trim(),
		address: input.address.trim(),
		updated_at: now,
	});

	return { id };
}

export function updateMockOffice(
	id: string,
	input: UpdateMockOfficeInput,
): MockOffice | null {
	const existing = getMockOfficeById(id);
	if (!existing) return null;

	const updatedOffice: MockOffice = {
		...existing,
		name: input.name.trim(),
		city: input.city.trim(),
		address: input.address.trim(),
		updated_at: new Date().toISOString(),
	};

	if (createdOffices.has(id)) {
		createdOffices.set(id, updatedOffice);
		return updatedOffice;
	}

	updatedOffices.set(id, updatedOffice);
	return updatedOffice;
}

export function deleteMockOffice(id: string): boolean {
	if (createdOffices.has(id)) {
		createdOffices.delete(id);
		return true;
	}

	if (updatedOffices.has(id)) {
		updatedOffices.delete(id);
		deletedOfficeIds.add(id);
		return true;
	}

	const existsInSeed = buildBaseOfficesMap().has(id);
	if (!existsInSeed) return false;

	deletedOfficeIds.add(id);
	return true;
}
