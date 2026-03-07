<script lang="ts">
	import type { PageData } from "./$types";
	import { page } from "$app/state";
	import { goto, invalidateAll } from "$app/navigation";
	import { _ } from "svelte-i18n";
	import ShipmentStatusBadge from "$lib/components/app/ShipmentStatusBadge.svelte";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";
	import {
		isKnownStatus,
		statusLabelKey,
		type ShipmentStatus,
	} from "$lib/domain/shipmentStatus";

	let { data }: { data: PageData } = $props();

	let lang = $derived(data.pathname.split("/")[1] || "en");
	let isRefreshing = $state(false);
	let filtersOpen = $state(false);

	let shipments = $derived(data.result.shipments);
	let officeLabelById = $derived(
		new Map(
			((data as { offices?: { id: string; name: string }[] }).offices ?? [])
				.filter((office) => Boolean(office?.id))
				.map((office) => [office.id, office.name] as const),
		),
	);
	let rawQuery = $derived(page.url.searchParams.get("q")?.trim() ?? "");
	let searchQuery = $derived(rawQuery.toLowerCase());
	let rawStatusFilter = $derived(
		page.url.searchParams.get("status") ?? "all",
	);
	let statusFilter = $derived.by(() => {
		const normalized = rawStatusFilter.toLowerCase();
		if (normalized === "all") return "all" as const;
		return isKnownStatus(normalized) ? normalized : ("all" as const);
	});

	let statusFilters = $derived<
		{ value: "all" | ShipmentStatus; label: string }[]
	>([
		{ value: "all", label: $_("all") },
		{ value: "new", label: $_(statusLabelKey("new")) },
		{ value: "accepted", label: $_(statusLabelKey("accepted")) },
		{ value: "pending", label: $_(statusLabelKey("pending")) },
		{ value: "in_transit", label: $_(statusLabelKey("in_transit")) },
		{ value: "delivered", label: $_(statusLabelKey("delivered")) },
		{ value: "cancelled", label: $_(statusLabelKey("cancelled")) },
	]);

	let filtered = $derived(
		shipments.filter((s) => {
			const shipmentOffice = displayOffice(s.office);
			const matchesSearch =
				!searchQuery ||
				s.id.toLowerCase().includes(searchQuery) ||
				shipmentOffice.toLowerCase().includes(searchQuery);
			const matchesStatus =
				statusFilter === "all" || s.status === statusFilter;
			return matchesSearch && matchesStatus;
		}),
	);

	function compactId(value: string): string {
		return `${value.slice(0, 8)}...`;
	}

	function displayOffice(office: string): string {
		return officeLabelById.get(office) ?? office;
	}

	function formatUpdated(iso: string): string {
		const dt = new Date(iso);
		return dt.toLocaleDateString(lang, {
			month: "short",
			day: "numeric",
			year: "numeric",
			hour: "2-digit",
			minute: "2-digit",
			hour12: false,
		});
	}

	async function handleRefresh() {
		isRefreshing = true;
		try {
			await invalidateAll();
		} finally {
			isRefreshing = false;
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest("[data-filters-menu]")) {
			filtersOpen = false;
		}
	}

	async function setStatusFilter(next: "all" | ShipmentStatus) {
		const url = new URL(page.url);
		if (next === "all") {
			url.searchParams.delete("status");
		} else {
			url.searchParams.set("status", next);
		}
		await goto(`${url.pathname}${url.search}`, {
			replaceState: true,
			keepFocus: true,
			noScroll: true,
		});
		filtersOpen = false;
	}
</script>

<svelte:document onclick={handleClickOutside} />

{#if data.result.state === "error"}
	<!-- Error state -->
	<div class="stagger stagger-1 flex flex-col items-center py-20 text-center">
		<div
			class="flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10"
		>
			<svg
				class="h-6 w-6 text-red-400"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
				/>
			</svg>
		</div>
		<h2 class="mt-4 text-lg font-semibold text-surface-50">
			{$_("shipments.error.headline")}
		</h2>
		<p class="mt-1 max-w-sm text-sm text-surface-400">
			{$_("shipments.error.hint")}
		</p>
		{#if data.result.message}
			<p class="mt-2 font-mono text-xs text-surface-600">
				{$_(data.result.message)}
			</p>
		{/if}
		<a
			href={`/${lang}/app/employee/shipments`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("shipments.retry")}
		</a>
	</div>
{:else if data.result.state === "empty"}
	<!-- Empty state -->
	<section
		class="stagger stagger-1 flex flex-col gap-1 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("shipments.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("shipments.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full bg-surface-800 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("shipments.viewing_office", {
						values: { office: data.activeOffice },
					})}
				</span>
			</div>
		</div>
		<a
			href={`/${lang}/app/employee/shipments/new`}
			class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("shipments.add")}
		</a>
	</section>

	<div
		class="stagger stagger-2 mt-6 flex flex-col items-center rounded-xl border border-surface-700/50 bg-surface-900 py-20 text-center"
	>
		<div
			class="flex h-12 w-12 items-center justify-center rounded-full bg-surface-800"
		>
			<svg
				class="h-6 w-6 text-surface-600"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m6 4.125l2.25 2.25m0 0l2.25-2.25M12 13.875V7.5M3.75 7.5h16.5"
				/>
			</svg>
		</div>
		<h2 class="mt-4 text-lg font-semibold text-surface-50">
			{$_("shipments.empty.headline")}
		</h2>
		<p class="mt-1 max-w-sm text-sm text-surface-400">
			{$_("shipments.empty.hint")}
		</p>
	</div>
{:else}
	<!-- OK state: header + table -->
	<section
		class="stagger stagger-1 relative z-20 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("shipments.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("shipments.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full bg-surface-800 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("shipments.viewing_office", {
						values: { office: data.activeOffice },
					})}
				</span>
			</div>
		</div>
		<div class="flex items-center gap-2">
			<a
				href={`/${lang}/app/employee/shipments/new`}
				class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("shipments.add")}
			</a>
				<div class="relative z-30" data-filters-menu>
				<button
					type="button"
					onclick={() => (filtersOpen = !filtersOpen)}
					class="cursor-pointer rounded-lg bg-surface-800 px-3 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-700 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
				>
					<span class="flex items-center gap-1.5">
						<svg
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M3 4h18M6 12h12M10 20h4"
							/>
						</svg>
						{$_("shipments.filters")}
					</span>
				</button>
				{#if filtersOpen}
					<div
						class="absolute right-0 top-full z-20 mt-2 w-56 rounded-lg border border-surface-700/70 bg-surface-900 p-1 shadow-lg shadow-black/30"
					>
						{#each statusFilters as filter (filter.value)}
							<button
								type="button"
								onclick={() => setStatusFilter(filter.value)}
								class={[
									"flex w-full cursor-pointer items-center justify-between rounded-md px-2.5 py-2 text-left text-sm transition-colors",
									statusFilter === filter.value
										? "bg-surface-800 text-surface-50"
										: "text-surface-400 hover:bg-surface-800 hover:text-surface-200",
								]}
							>
								<span>{filter.label}</span>
								{#if statusFilter === filter.value}
									<svg
										class="h-4 w-4 text-accent"
										viewBox="0 0 20 20"
										fill="currentColor"
									>
										<path
											fill-rule="evenodd"
											d="M16.704 5.29a1 1 0 010 1.42l-8 8a1 1 0 01-1.42 0l-4-4a1 1 0 111.42-1.42L8 12.59l7.29-7.3a1 1 0 011.414 0z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>
			<button
				onclick={handleRefresh}
				aria-label={$_("shipments.refresh")}
				class="cursor-pointer rounded-lg bg-surface-800 p-2 text-surface-400 transition-colors hover:bg-surface-700"
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

	<div
		class="stagger stagger-2 relative z-0 mt-4 overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900"
	>
		<div class="overflow-x-auto">
			<table class="w-full min-w-[540px]">
				<thead>
					<tr>
						<th
							class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipments.col.id")}
						</th>
						<th
							class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipments.col.status")}
						</th>
						<th
							class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipments.col.office")}
						</th>
						<th
							class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipments.col.updated")}
						</th>
						<th class="w-10 px-5 py-3"></th>
					</tr>
				</thead>
				<tbody>
					{#if filtered.length === 0}
						<tr>
							<td
								colspan="5"
								class="py-12 text-center text-sm text-surface-600"
							>
								{$_("shipments.no_results")}
							</td>
						</tr>
					{:else}
						{#each filtered as shipment (shipment.id)}
							<tr
								onclick={() =>
									goto(
										`/${lang}/app/employee/shipments/${shipment.id}`,
									)}
								class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50 focus-visible:bg-surface-800/50 focus-visible:outline-none"
								tabindex="0"
								role="link"
								onkeydown={(e) => {
									if (e.key === "Enter" || e.key === " ") {
										e.preventDefault();
										goto(
											`/${lang}/app/employee/shipments/${shipment.id}`,
										);
									}
								}}
							>
								<td class="px-5 py-3 text-sm text-accent">
									<div class="flex items-center gap-2">
										<span class="font-mono"
											>{compactId(shipment.id)}</span
										>
										<CopyIconButton
											value={shipment.id}
											title={$_("shipments.copy_id")}
											ariaLabel={$_("shipments.copy_id")}
											stopPropagation
										/>
									</div>
								</td>
								<td class="px-5 py-3">
									<ShipmentStatusBadge
										status={shipment.status}
									/>
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{displayOffice(shipment.office)}
								</td>
								<td class="px-5 py-3 text-sm text-surface-400">
									{formatUpdated(shipment.updatedAt)}
								</td>
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
		</div>
	</div>
{/if}

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
</style>
