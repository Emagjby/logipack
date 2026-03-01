export class HubApiMappingError extends Error {
	endpoint: string;
	field: string;
	cause?: unknown;

	constructor(args: {
		endpoint: string;
		field: string;
		message: string;
		cause?: unknown;
	}) {
		super(args.message);
		this.name = "HubApiMappingError";
		this.endpoint = args.endpoint;
		this.field = args.field;
		this.cause = args.cause;
	}
}

function isRecord(v: unknown): v is Record<string, unknown> {
	return !!v && typeof v === "object" && !Array.isArray(v);
}

export function requireRecord(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): Record<string, unknown> {
	if (!isRecord(args.value)) {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected object for ${args.field}`,
		});
	}
	return args.value;
}

export function requireString(args: {
	endpoint: string;
	field: string;
	value: unknown;
	trim?: boolean;
	nonEmpty?: boolean;
}): string {
	const { endpoint, field, value } = args;
	if (typeof value !== "string") {
		throw new HubApiMappingError({
			endpoint,
			field,
			message: `expected string for ${field}`,
		});
	}

	const s = args.trim === false ? value : value.trim();
	if (args.nonEmpty && s.length === 0) {
		throw new HubApiMappingError({
			endpoint,
			field,
			message: `expected non-empty string for ${field}`,
		});
	}

	return s;
}

export function cleanNullableString(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): string | null {
	const { endpoint, field, value } = args;
	if (value === null || value === undefined) return null;
	if (typeof value !== "string") {
		throw new HubApiMappingError({
			endpoint,
			field,
			message: `expected string or null for ${field}`,
		});
	}
	const s = value.trim();
	return s.length === 0 ? null : s;
}

export function requireIsoDateTime(args: {
	endpoint: string;
	field: string;
	value: unknown;
}): string {
	const s = requireString({
		endpoint: args.endpoint,
		field: args.field,
		value: args.value,
		trim: true,
		nonEmpty: true,
	});

	if (!s.includes("T")) {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected ISO datetime for ${args.field}`,
		});
	}

	const ms = Date.parse(s);
	if (!Number.isFinite(ms)) {
		throw new HubApiMappingError({
			endpoint: args.endpoint,
			field: args.field,
			message: `expected ISO datetime for ${args.field}`,
		});
	}

	return s;
}
