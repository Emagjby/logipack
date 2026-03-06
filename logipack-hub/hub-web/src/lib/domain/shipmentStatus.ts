/**
 * Shared shipment-status abstraction.
 *
 * Single source of truth for status type, i18n label keys,
 * badge styling, and normalisation logic.
 * Used by the shipments list page, future detail page, and dashboard.
 */

// ── Canonical status union ──────────────────────────────────────────
export const SHIPMENT_STATUSES = [
	"new",
	"accepted",
	"pending",
	"in_transit",
	"delivered",
	"cancelled",
] as const;

export type ShipmentStatus = (typeof SHIPMENT_STATUSES)[number];

// ── Normalisation ───────────────────────────────────────────────────

const ALIAS_MAP: Record<string, ShipmentStatus> = {
	new: "new",
	processed: "pending",
	accepted: "accepted",
	processing: "pending",
	pending: "pending",
	in_transit: "in_transit",
	delivered: "delivered",
	cancelled: "cancelled",
	created: "new",
	intransit: "in_transit",
};

function normalizeStatusInput(raw: string): string {
	return raw.trim().toLowerCase().replace(/[\s-]+/g, "_");
}

/** Coerce an arbitrary string into a known status or `"unknown"`. */
export function normalizeShipmentStatus(
	raw: string,
): ShipmentStatus | "unknown" {
	const normalizedRaw = normalizeStatusInput(raw);
	return ALIAS_MAP[normalizedRaw] ??
		(isKnownStatus(normalizedRaw) ? normalizedRaw : "unknown");
}

export function isKnownStatus(value: string): value is ShipmentStatus {
	return (SHIPMENT_STATUSES as readonly string[]).includes(value);
}

// ── i18n label key mapping ──────────────────────────────────────────

const LABEL_KEYS: Record<ShipmentStatus | "unknown", string> = {
	new: "shipment_status.new",
	accepted: "shipment_status.accepted",
	pending: "shipment_status.pending",
	in_transit: "shipment_status.in_transit",
	delivered: "shipment_status.delivered",
	cancelled: "shipment_status.cancelled",
	unknown: "shipment_status.unknown",
};

/** Return the i18n key for a status label (e.g. `"shipment_status.in_transit"`). */
export function statusLabelKey(
	status: ShipmentStatus | "unknown",
): string {
	return LABEL_KEYS[status] ?? LABEL_KEYS.unknown;
}

// ── Badge class mapping (uses existing surface/accent tokens) ───────

const BADGE_CLASSES: Record<ShipmentStatus | "unknown", string> = {
	new: "bg-slate-500/10 text-slate-300",
	accepted: "bg-cyan-500/10 text-cyan-300",
	pending: "bg-amber-500/10 text-amber-400",
	in_transit: "bg-indigo-500/10 text-indigo-300",
	delivered: "bg-emerald-500/10 text-emerald-300",
	cancelled: "bg-red-500/10 text-red-400",
	unknown: "bg-surface-700/50 text-surface-600",
};

/** Tailwind classes for a status badge pill. */
export function statusBadgeClass(
	status: ShipmentStatus | "unknown",
): string {
	return BADGE_CLASSES[status] ?? BADGE_CLASSES.unknown;
}

const DOT_CLASSES: Record<ShipmentStatus | "unknown", string> = {
	new: "bg-slate-300/80",
	accepted: "bg-cyan-300/80",
	pending: "bg-amber-400/80",
	in_transit: "bg-indigo-300/80",
	delivered: "bg-emerald-300/80",
	cancelled: "bg-red-400/80",
	unknown: "bg-surface-600",
};

/** Tailwind classes for a status dot indicator. */
export function statusDotClass(status: ShipmentStatus | "unknown"): string {
	return DOT_CLASSES[status] ?? DOT_CLASSES.unknown;
}

// ── Row type used by the shipments list ─────────────────────────────

export interface ShipmentRow {
	id: string;
	status: ShipmentStatus | "unknown";
	office: string;
	updatedAt: string; // ISO-8601
}
