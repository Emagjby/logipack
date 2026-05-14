<script lang="ts">
	import { _ } from "svelte-i18n";
	import {
		normalizeShipmentStatus,
		statusBadgeClass,
		statusLabelKey,
		type ShipmentStatus,
	} from "$lib/domain/shipmentStatus";

	let {
		status,
		compact = false,
		extraClass = "",
	}: {
		status: string | ShipmentStatus | "unknown";
		compact?: boolean;
		extraClass?: string;
	} = $props();

	let normalized = $derived(
		status === "unknown"
			? "unknown"
			: normalizeShipmentStatus(String(status)),
	);
</script>

<span
	class={[
		compact
			? "inline-flex whitespace-nowrap rounded px-1.5 py-0.5 text-[10px] font-medium"
			: "inline-flex whitespace-nowrap rounded-full px-2 py-0.5 text-xs font-medium",
		statusBadgeClass(normalized),
		extraClass,
	]}
>
	{$_(statusLabelKey(normalized))}
</span>
