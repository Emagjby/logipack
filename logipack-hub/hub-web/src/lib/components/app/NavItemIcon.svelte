<script lang="ts">
	import Building2 from "@lucide/svelte/icons/building-2";
	import ClipboardCheck from "@lucide/svelte/icons/clipboard-check";

	type IconName =
		| "dashboard"
		| "shipments"
		| "reports"
		| "clients"
		| "offices"
		| "employees"
		| "audit"
		| "profile";

	let {
		name,
		active = false,
	}: {
		name: IconName;
		active?: boolean;
	} = $props();

	let iconClass = $derived(
		[
			"h-[18px] w-[18px] shrink-0",
			active ? "text-accent" : "text-surface-600 group-hover:text-surface-400",
		].join(" "),
	);
</script>

{#if name === "dashboard"}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
	>
		<rect x="3" y="3" width="6" height="6" rx="1" />
		<rect x="11" y="3" width="6" height="6" rx="1" />
		<rect x="3" y="11" width="6" height="6" rx="1" />
		<rect x="11" y="11" width="6" height="6" rx="1" />
	</svg>
{:else if name === "shipments"}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
	>
		<path d="M3 7l7-4 7 4v6l-7 4-7-4V7z" />
		<path d="M3 7l7 4m0 0l7-4m-7 4v7" />
	</svg>
{:else if name === "reports"}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<path d="M4 16V9" />
		<path d="M10 16V4" />
		<path d="M16 16v-6" />
		<path d="M3 16h14" />
	</svg>
{:else if name === "clients"}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<circle cx="10" cy="7" r="3" />
		<path d="M4 16a6 6 0 0112 0" />
	</svg>
{:else if name === "offices"}
	<!-- size=18 is explicit; strokeWidth 1.7 intentionally matches visual weight of inline 1.5 SVG icons. -->
	<Building2
		class={iconClass}
		size={18}
		strokeWidth={1.7}
	/>
{:else if name === "employees"}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<circle cx="7" cy="7" r="2.5" />
		<circle cx="13.5" cy="8" r="2" />
		<path d="M2.5 16a4.5 4.5 0 019 0" />
		<path d="M11 16a3.2 3.2 0 016.4 0" />
	</svg>
{:else if name === "audit"}
	<!-- size=18 is explicit; strokeWidth 1.7 intentionally matches visual weight of inline 1.5 SVG icons. -->
	<ClipboardCheck
		class={iconClass}
		size={18}
		strokeWidth={1.7}
	/>
{:else}
	<svg
		class={iconClass}
		viewBox="0 0 20 20"
		fill="none"
		stroke="currentColor"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<circle cx="10" cy="7" r="3" />
		<path d="M4.5 16a5.5 5.5 0 0111 0" />
	</svg>
{/if}
