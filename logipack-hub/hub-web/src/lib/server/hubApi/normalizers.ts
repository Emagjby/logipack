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

export function requireIsoDateTime(_args: {
	endpoint: string;
	field: string;
	value: unknown;
}): string {
	throw new HubApiMappingError({
		endpoint: _args.endpoint,
		field: _args.field,
		message: "requireIsoDateTime not implemented",
	});
}

export function cleanNullablestring(_v: unknown): string | null {
	return null;
}

export function normalizeShipmentStatusStrict(_args: {
	endpoint: string;
	field: string;
	value: unknown;
}): "unknown" | string {
	return "unknown";
}
