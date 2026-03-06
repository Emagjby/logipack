export interface StrataPackage {
	hash: string;
	prev_hash: string | null;
	stream_id: string;
	seq: number;
	event_type: string;
	created_at: string;
	payload_json: unknown;
}

export function formatDateTime(iso: string, lang: string): string {
	return new Date(iso).toLocaleDateString(lang, {
		month: "short",
		day: "numeric",
		year: "numeric",
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
	});
}

export function shortenHash(h: string): string {
	if (h.length <= 16) return h;
	return h.slice(0, 8) + "\u2026" + h.slice(-4);
}

export function formatEventType(type: string): string {
	const normalized = type.trim();
	const lower = normalized.toLowerCase().replace(/[_\s]+/g, "-");

	if (lower === "status-change" || lower === "statuschanged") {
		return "Status Change";
	}

	const words = normalized.match(/[_-]/)
		? normalized.split(/[_-]+/)
		: normalized.replace(/([a-z])([A-Z])/g, "$1 $2").split(" ");

	return words
		.map((word, i) =>
			i === 0
				? word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()
				: word.toLowerCase(),
		)
		.join(" ");
}
