<script lang="ts">
	import type { PageData } from "./$types";
	import { goto } from "$app/navigation";
	import { _ } from "svelte-i18n";
	import ShipmentStatusBadge from "$lib/components/app/ShipmentStatusBadge.svelte";
	import { compactId } from "$lib/utils/idDisplay";
	import {
		normalizeShipmentStatus,
		statusDotClass,
	} from "$lib/domain/shipmentStatus";

	let { data }: { data: PageData } = $props();

	let lang = $derived(data.pathname.split("/")[1] || "en");

	let isRefreshing = $state(false);
	let lookupValue = $state("");
	let shipmentFilter = $state<"all" | "in-transit" | "pending" | "delivered">(
		"all",
	);

	let filteredShipments = $derived(
		shipmentFilter === "all"
			? data.recentShipments
			: data.recentShipments.filter((s) => s.status === shipmentFilter),
	);

	let minutesAgo = $derived.by(() => {
		if (typeof globalThis.window === "undefined")
			return $_("common.just_now");
		const diff = Date.now() - new Date(data.lastRefresh).getTime();
		const mins = Math.max(0, Math.floor(diff / 60000));
		if (mins === 0) return $_("common.just_now");
		return $_("common.minutes_ago", { values: { minutes: mins } });
	});

	function handleRefresh() {
		isRefreshing = true;
		setTimeout(() => {
			isRefreshing = false;
		}, 600);
	}

	function handleLookup() {
		const v = lookupValue.trim();
		if (v) goto(`/${lang}/app/employee/shipments/${v}`);
	}

	function sparklinePoints(values: number[]): { line: string; area: string } {
		if (values.length === 0) return { line: "", area: "" };
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min || 1;
		const step = 100 / (values.length - 1);
		const pts = values.map((v, i) => {
			const x = i * step;
			const y = 28 - ((v - min) / range) * 24 + 2;
			return `${x},${y}`;
		});
		const line = pts.join(" ");
		const area = `M${pts[0]} ${pts
			.slice(1)
			.map((p) => `L${p}`)
			.join(" ")} L100,30 L0,30 Z`;
		return { line, area };
	}

	let filterOptions = $derived<
		{
			label: string;
			value: "all" | "in-transit" | "pending" | "delivered";
		}[]
	>([
		{ label: $_("all"), value: "all" },
		{ label: $_("in-transit"), value: "in-transit" },
		{ label: $_("pending"), value: "pending" },
		{ label: $_("delivered"), value: "delivered" },
	]);

	function dotColor(tag: string | null): string {
		if (!tag) return "bg-surface-600";
		return statusDotClass(normalizeShipmentStatus(tag));
	}
</script>

<section
	class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
>
	<div>
		<h1 class="text-2xl font-bold text-surface-50">{data.greeting}</h1>
		<p class="mt-1 text-sm text-surface-400">
			{$_("empd.subheadline")}
		</p>
		<div class="mt-2 flex items-center gap-3">
			<span
				class="flex items-center gap-1.5 rounded-full bg-accent/10 px-2 py-0.5 text-[11px] font-semibold text-accent"
			>
				<span class="pulse-dot h-1.5 w-1.5 rounded-full bg-accent"
				></span>
				{$_("live")}
			</span>
			<span class="text-[11px] text-surface-600"
				>{$_("updated")} {minutesAgo}</span
			>
		</div>
	</div>
	<div class="flex items-center gap-2">
		<a
			href={`/${lang}/app/employee/shipments`}
			class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("empd.view_shipments")}
		</a>
		{#if data.canCreateShipment}
			<a
				href={`/${lang}/app/employee/shipments/new`}
				class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("empd.create_shipment")}
			</a>
		{/if}
		<button
			onclick={handleRefresh}
			aria-label="Refresh"
			class="rounded-lg bg-surface-800 p-2 text-surface-400 transition-colors hover:bg-surface-700"
		>
			<svg
				class={["h-5 w-5", isRefreshing && "animate-spin"]}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
				/>
			</svg>
		</button>
	</div>
</section>

<section class="stagger stagger-2 mt-6 grid grid-cols-1 gap-4 md:grid-cols-3">
	{#each data.kpis as kpi (kpi.label)}
		{@const sp = sparklinePoints(kpi.sparkline)}
		<a
			href={`/${lang}/app/employee/shipments`}
			class="group relative block cursor-pointer overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900 p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-surface-600/50 hover:shadow-lg hover:shadow-black/20"
		>
			<div class="flex items-center justify-between">
				<span
					class="text-xs font-medium uppercase tracking-wider text-surface-400"
					>{kpi.label}</span
				>
				<span
					class={[
						"h-2 w-2 rounded-full",
						kpi.severity === "good" ? "bg-accent" : "bg-amber-400",
					]}
				></span>
			</div>
			<div class="mt-1 text-3xl font-bold text-surface-50">
				{kpi.value}
			</div>
			<div class="mt-1 flex items-center gap-1.5">
				{#if kpi.trend === "up"}
					<svg
						class="h-3 w-3 text-accent"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M5 15l7-7 7 7"
						/>
					</svg>
					<span class="text-xs font-medium text-accent"
						>{kpi.change}</span
					>
				{:else}
					<svg
						class="h-3 w-3 text-amber-400"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M5 12h14"
						/>
					</svg>
					<span class="text-xs font-medium text-amber-400"
						>{kpi.change}</span
					>
				{/if}
			</div>
			<div class="mt-1 text-[11px] text-surface-600">{kpi.context}</div>
			<svg
				class={[
					"mt-3 h-8 w-full",
					kpi.severity === "good"
						? "text-accent/40"
						: "text-amber-400/40",
				]}
				viewBox="0 0 100 32"
				preserveAspectRatio="none"
			>
				<defs>
					<linearGradient
						id="grad-{kpi.label}"
						x1="0"
						y1="0"
						x2="0"
						y2="1"
					>
						<stop
							offset="0%"
							stop-color="currentColor"
							stop-opacity="0.15"
						/>
						<stop
							offset="100%"
							stop-color="currentColor"
							stop-opacity="0"
						/>
					</linearGradient>
				</defs>
				<path d={sp.area} fill="currentColor" />
				<polyline
					points={sp.line}
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			</svg>
			{#if isRefreshing}
				<div
					class="absolute inset-0 animate-pulse rounded-xl bg-surface-800/80"
				></div>
			{/if}
		</a>
	{/each}
</section>

<section
	class="stagger stagger-3 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-4"
>
	<div class="flex items-center gap-3">
		<svg
			class="h-4 w-4 shrink-0 text-surface-600"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
			/>
		</svg>
		<span class="shrink-0 text-sm font-medium text-surface-200"
			>{$_("empd.quick_lookup")}</span
		>
		<input
			type="text"
			bind:value={lookupValue}
			placeholder={$_("empd.search_placeholder")}
			onkeydown={(e: KeyboardEvent) => {
				if (e.key === "Enter") handleLookup();
			}}
			class="flex-1 rounded-lg border border-surface-700 bg-surface-800 px-3 py-2 text-sm text-surface-50 placeholder-surface-600 focus:outline-none focus:ring-1 focus:ring-accent/50"
		/>
		<button
			disabled={lookupValue.trim() === ""}
			onclick={handleLookup}
			class={[
				"rounded-lg px-3 py-2 text-sm font-medium transition-colors",
				lookupValue.trim()
					? "bg-accent text-surface-950 hover:bg-accent-hover cursor-pointer"
					: "bg-surface-800 text-surface-600 cursor-not-allowed",
			]}
		>
			{$_("track")}
		</button>
		{#if lookupValue.trim()}
			<span class="text-[11px] text-surface-600">{$_("press_enter")}</span
			>
		{/if}
	</div>
	{#if !lookupValue.trim() && data.recentSearches.length > 0}
		<div
			class="mt-3 flex items-center gap-2 border-t border-surface-800 pt-3"
		>
			<span class="text-[11px] text-surface-600">{$_("recent")}</span>
			{#each data.recentSearches as id (id)}
				<button
					onclick={() => {
						lookupValue = id;
					}}
					title={id}
					class="cursor-pointer rounded-md bg-surface-800 px-2 py-1 font-mono text-xs text-surface-400 transition-colors hover:bg-surface-700 hover:text-surface-200"
				>
					{compactId(id)}
				</button>
			{/each}
		</div>
	{/if}
</section>

<div class="stagger stagger-4 mt-6 grid grid-cols-1 gap-4 lg:grid-cols-5">
	<div
		class="relative overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900 lg:col-span-3"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<div class="flex items-center justify-between">
				<span class="text-sm font-semibold text-surface-50"
					>{$_("recent_shipments")}</span
				>
				<a
					href={`/${lang}/app/employee/shipments`}
					class="text-xs text-accent transition-colors hover:text-accent-hover"
					>{$_("view_all")}</a
				>
			</div>
			<div class="mt-3 flex items-center gap-1.5">
				{#each filterOptions as opt (opt.value)}
					<button
						onclick={() => {
							shipmentFilter = opt.value;
						}}
						class={[
							"cursor-pointer rounded-full px-2.5 py-1 text-[11px] font-medium transition-colors",
							shipmentFilter === opt.value
								? "bg-accent/10 text-accent"
								: "text-surface-600 hover:bg-surface-800 hover:text-surface-400",
						]}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>
		<div class="relative">
			<table class="w-full">
				<thead>
					<tr>
						<th
							class="w-8 px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						></th>
						<th
							class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>ID</th
						>
						<th
							class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>{$_("destination")}</th
						>
						<th
							class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>{$_("status")}</th
						>
						<th
							class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>{$_("eta")}</th
						>
						<th
							class="w-10 px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						></th>
					</tr>
				</thead>
				<tbody>
					{#if filteredShipments.length === 0}
						<tr>
							<td
								colspan="6"
								class="py-8 text-center text-sm text-surface-600"
							>
								{$_("empd.no_shipments_for_filter")}
								<a
									href={`/${lang}/app/employee/shipments`}
									class="ml-1 text-accent hover:text-accent-hover"
									>{$_("empd.view_all_shipments")}</a
								>
							</td>
						</tr>
					{:else}
						{#each filteredShipments as shipment (shipment.id)}
							<tr
								onclick={() =>
									goto(
										`/${lang}/app/employee/shipments/${shipment.id}`,
									)}
								class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50"
							>
								<td class="px-5 py-3">
									{#if shipment.priority === "high"}
										<span
											class="block h-1.5 w-1.5 rounded-full bg-amber-400"
										></span>
									{/if}
								</td>
								<td
									class="px-5 py-3 font-mono text-sm text-accent"
									title={shipment.id}
									>{compactId(shipment.id)}</td
								>
								<td class="px-5 py-3 text-sm text-surface-200"
									>{shipment.destination}</td
								>
								<td class="px-5 py-3">
									<ShipmentStatusBadge status={shipment.status} />
								</td>
								<td class="px-5 py-3 text-sm text-surface-400"
									>{shipment.eta}</td
								>
								<td class="px-5 py-3">
									<svg
										class="h-4 w-4 text-surface-600 transition-colors group-hover:text-surface-400"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
										stroke-width="2"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											d="M9 5l7 7-7 7"
										/>
									</svg>
								</td>
							</tr>
						{/each}
					{/if}
				</tbody>
			</table>
			{#if isRefreshing}
				<div
					class="absolute inset-0 animate-pulse bg-surface-800/80"
				></div>
			{/if}
		</div>
	</div>

	<div
		class="stagger stagger-5 rounded-xl border border-surface-700/50 bg-surface-900 lg:col-span-2"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<span class="text-sm font-semibold text-surface-50"
				>{$_("activity")}</span
			>
		</div>
		{#if data.activity.length === 0}
			<div class="px-5 py-8 text-center text-sm text-surface-600">
				{$_("no_recent_activity")}
			</div>
		{:else}
			{#each data.activity as group, i (group.group)}
				<div class={["", i > 0 && "mt-1 border-t border-surface-800"]}>
					<div class="px-5 pb-1 pt-3">
						<span
							class="text-[10px] font-medium uppercase tracking-widest text-surface-600"
							>{group.group}</span
						>
					</div>
					{#each group.items as item (item.shipmentId + item.time)}
						<div class="flex gap-3 px-5 py-3">
							<span
								class={[
									"mt-1.5 h-2 w-2 shrink-0 rounded-full",
									dotColor(item.tag),
								]}
							></span>
							<div class="min-w-0 flex-1">
								<div class="text-sm text-surface-200">
									<span class="font-mono text-accent" title={item.shipmentId}
										>{compactId(item.shipmentId)}</span
									>
									{item.title}
								</div>
								<div class="mt-1 flex items-center gap-2">
									<span class="text-[11px] text-surface-600"
										>{item.time}</span
									>
									{#if item.tag}
										<ShipmentStatusBadge
											status={item.tag}
											compact={true}
										/>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/each}
		{/if}
		<div class="border-t border-surface-800 px-5 py-3 text-center">
			<button
				class="cursor-pointer text-xs text-surface-600 transition-colors hover:text-surface-400"
				>{$_("show-more")}</button
			>
		</div>
	</div>
</div>

<style>
	@keyframes fadeSlideUp {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.stagger {
		animation: fadeSlideUp 0.4s ease-out both;
	}
	.stagger-1 {
		animation-delay: 0.05s;
	}
	.stagger-2 {
		animation-delay: 0.1s;
	}
	.stagger-3 {
		animation-delay: 0.15s;
	}
	.stagger-4 {
		animation-delay: 0.2s;
	}
	.stagger-5 {
		animation-delay: 0.25s;
	}

	@keyframes pulse-dot {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.5;
			transform: scale(1.5);
		}
	}

	.pulse-dot {
		animation: pulse-dot 2s ease-in-out infinite;
	}
</style>
