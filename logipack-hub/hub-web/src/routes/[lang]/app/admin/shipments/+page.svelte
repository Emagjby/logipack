<script lang="ts">
	import { browser } from "$app/environment";
	import { goto, invalidateAll } from "$app/navigation";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import ShipmentStatusBadge from "$lib/components/app/ShipmentStatusBadge.svelte";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";
	import {
		SHIPMENT_STATUSES,
		isKnownStatus,
		statusLabelKey,
		type ShipmentStatus,
	} from "$lib/domain/shipmentStatus";
	import type { PageData } from "./$types";

	type FilterTab = "office" | "status";

	let { data }: { data: PageData } = $props();

	let filtersOpen = $state(false);
	let isRefreshing = $state(false);
	let activeTab = $state<FilterTab>("office");
	let draftOffice = $state<string | null>(null);
	let draftStatuses = $state<ShipmentStatus[]>([]);
	let filtersButtonEl = $state<HTMLButtonElement | null>(null);
	let filtersDialogEl = $state<HTMLDivElement | null>(null);
	let lastFocusedEl = $state<HTMLElement | null>(null);

	let lang = $derived(data.pathname.split("/")[1] || "en");
	let shipments = $derived(
		data.result.state === "ok" ? data.result.shipments : [],
	);
	let officeLabelById = $derived(
		new Map(
			((data as { offices?: { id: string; name: string }[] }).offices ?? [])
				.filter((office) => Boolean(office?.id))
				.map((office) => [office.id, office.name] as const),
		),
	);
	let offices = $derived(
		[...new Set(shipments.map((shipment) => displayOffice(shipment.office)))].sort(
			(a, b) => a.localeCompare(b),
		),
	);
	let officeCount = $derived(offices.length);
	// Query-param search is intentionally URL-driven for now (no local input in this view).
	let searchQuery = $derived(
		page.url.searchParams.get("q")?.trim().toLowerCase() ?? "",
	);
	let appliedOffice = $derived.by(() => {
		const rawOffice = page.url.searchParams.get("office")?.trim();
		return rawOffice && rawOffice.length > 0 ? rawOffice : null;
	});
	let appliedStatuses = $derived.by(() =>
		parseStatusParam(page.url.searchParams.get("status")),
	);
	let filtered = $derived(
		shipments.filter((shipment) => {
			const shipmentOffice = displayOffice(shipment.office);
			const matchesSearch =
				!searchQuery ||
				shipment.id.toLowerCase().includes(searchQuery) ||
				shipmentOffice.toLowerCase().includes(searchQuery);
			const matchesOffice =
				!appliedOffice || shipmentOffice === appliedOffice;
			const matchesStatus =
				appliedStatuses.length === 0 ||
				(appliedStatuses as readonly string[]).includes(shipment.status);
			return matchesSearch && matchesOffice && matchesStatus;
		}),
	);

	function compactId(value: string): string {
		return `${value.slice(0, 8)}...`;
	}

	function displayOffice(office: string): string {
		return officeLabelById.get(office) ?? office;
	}

	$effect(() => {
		if (!browser) return;

		if (filtersOpen) {
			lastFocusedEl = document.activeElement as HTMLElement | null;
			document.body.classList.add("overflow-hidden");
			queueMicrotask(() => {
				focusFirstInDialog();
			});
			return () => {
				document.body.classList.remove("overflow-hidden");
			};
		}

		document.body.classList.remove("overflow-hidden");
	});

	function parseStatusParam(raw: string | null): ShipmentStatus[] {
		if (!raw) return [];

		const selected: ShipmentStatus[] = [];
		for (const token of raw.split(",")) {
			const normalized = token.trim().toLowerCase();
			if (isKnownStatus(normalized) && !selected.includes(normalized)) {
				selected.push(normalized);
			}
		}

		return SHIPMENT_STATUSES.filter((status) => selected.includes(status));
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

	function openShipment(id: string): void {
		void goto(`/${lang}/app/admin/shipments/${id}`);
	}

	function openNewShipment(): void {
		void goto(`/${lang}/app/admin/shipments/new`);
	}

	function handleRowKeydown(event: KeyboardEvent, shipmentId: string): void {
		if (event.key === "Enter" || event.key === " ") {
			event.preventDefault();
			openShipment(shipmentId);
		}
	}

	function syncDraftWithApplied(): void {
		draftOffice = appliedOffice;
		draftStatuses = [...appliedStatuses];
	}

	function openFilters(): void {
		syncDraftWithApplied();
		activeTab = "office";
		filtersOpen = true;
	}

	function closeFilters(): void {
		filtersOpen = false;
		queueMicrotask(() => {
			(lastFocusedEl ?? filtersButtonEl)?.focus();
		});
	}

	function cancelFilters(): void {
		syncDraftWithApplied();
		closeFilters();
	}

	function handleDocumentKeydown(event: KeyboardEvent): void {
		if (!filtersOpen) return;
		if (event.key === "Escape") {
			event.preventDefault();
			cancelFilters();
			return;
		}
		if (event.key === "Tab") {
			trapDialogFocus(event);
		}
	}

	function getDialogFocusableElements(): HTMLElement[] {
		if (!filtersDialogEl) return [];
		const selectors = [
			"button:not([disabled])",
			"[href]",
			"input:not([disabled])",
			"select:not([disabled])",
			"textarea:not([disabled])",
			"[tabindex]:not([tabindex='-1'])",
		];
		return Array.from(
			filtersDialogEl.querySelectorAll<HTMLElement>(selectors.join(",")),
		).filter((el) => !el.hasAttribute("hidden"));
	}

	function focusFirstInDialog(): void {
		const focusable = getDialogFocusableElements();
		if (focusable.length > 0) {
			focusable[0]?.focus();
			return;
		}
		filtersDialogEl?.focus();
	}

	function trapDialogFocus(event: KeyboardEvent): void {
		const focusable = getDialogFocusableElements();
		if (focusable.length === 0) {
			event.preventDefault();
			filtersDialogEl?.focus();
			return;
		}

		const first = focusable[0]!;
		const last = focusable[focusable.length - 1]!;
		const active = document.activeElement as HTMLElement | null;

		if (event.shiftKey) {
			if (active === first || !filtersDialogEl?.contains(active)) {
				event.preventDefault();
				last.focus();
			}
			return;
		}

		if (active === last) {
			event.preventDefault();
			first.focus();
		}
	}

	function setDraftOffice(office: string | null): void {
		draftOffice = office;
	}

	function clearDraftStatuses(): void {
		draftStatuses = [];
	}

	function resetDraftFilters(): void {
		draftOffice = null;
		draftStatuses = [];
	}

	function toggleDraftStatus(status: ShipmentStatus): void {
		const selected = [...draftStatuses];
		if (selected.includes(status)) {
			draftStatuses = selected.filter((item) => item !== status);
		} else {
			draftStatuses = [...selected, status];
		}

		draftStatuses = SHIPMENT_STATUSES.filter((item) =>
			draftStatuses.includes(item),
		);
	}

	async function applyFilters(): Promise<void> {
		const url = new URL(page.url);

		if (draftOffice) {
			url.searchParams.set("office", draftOffice);
		} else {
			url.searchParams.delete("office");
		}

		if (draftStatuses.length > 0) {
			url.searchParams.set("status", draftStatuses.join(","));
		} else {
			url.searchParams.delete("status");
		}

		await goto(`${url.pathname}${url.search}`, {
			replaceState: true,
			keepFocus: true,
			noScroll: true,
		});
		closeFilters();
	}

	async function handleRefresh() {
		isRefreshing = true;
		try {
			await invalidateAll();
		} finally {
			isRefreshing = false;
		}
	}
</script>

<svelte:document onkeydown={handleDocumentKeydown} />

{#if data.result.state === "error"}
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
			href={`/${lang}/app/admin/shipments`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("shipments.retry")}
		</a>
	</div>
{:else}
	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.shipments.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.shipments.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("admin.shipments.scope", {
						values: { count: officeCount },
					})}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={openNewShipment}
				class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.shipments.new_shipment")}
			</button>
			{#if data.result.state === "ok"}
				<button
					bind:this={filtersButtonEl}
					type="button"
					onclick={openFilters}
					aria-expanded={filtersOpen}
					aria-controls="admin-shipments-filters-dialog"
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
			{/if}
		</div>
	</section>

	{#if data.result.state === "empty"}
		<div
			class="stagger stagger-2 mt-6 flex flex-col items-center rounded-xl border border-surface-700/50 bg-surface-900 py-20 text-center"
		>
			<div
				class="flex h-12 w-12 items-center justify-center rounded-full bg-surface-800"
			>
				<svg
					class="h-6 w-6 text-surface-500"
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
				{$_("admin.shipments.empty.headline")}
			</h2>
			<p class="mt-1 max-w-sm text-sm text-surface-400">
				{$_("admin.shipments.empty.hint")}
			</p>
			<button
				type="button"
				onclick={openNewShipment}
				class="mt-5 cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.shipments.new_shipment")}
			</button>
		</div>
	{:else}
		<div
			class="stagger stagger-2 mt-4 overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900"
		>
			<div class="overflow-x-auto">
				<table class="w-full min-w-[560px]">
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
									onclick={() => openShipment(shipment.id)}
									onkeydown={(event) =>
										handleRowKeydown(event, shipment.id)}
									class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50 focus-visible:bg-surface-800/50 focus-visible:outline-none"
									tabindex="0"
									role="link"
								>
							<td class="px-5 py-3 text-sm text-accent">
								<div class="flex items-center gap-2">
									<span class="font-mono">{compactId(shipment.id)}</span>
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
							<td
								class="px-5 py-3 text-sm text-surface-200"
							>
								{displayOffice(shipment.office)}
							</td>
									<td
										class="px-5 py-3 text-sm text-surface-400"
									>
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
{/if}

{#if data.result.state === "ok" && filtersOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4">
		<button
			type="button"
			aria-label={$_("admin.shipments.filters.close")}
			onclick={cancelFilters}
			class="absolute inset-0 bg-surface-950/55 backdrop-blur-sm"
		></button>
		<div
			bind:this={filtersDialogEl}
			id="admin-shipments-filters-dialog"
			role="dialog"
			aria-modal="true"
			aria-labelledby="admin-shipments-filters-title"
			tabindex="-1"
			class="relative flex max-h-[calc(100vh-2rem)] w-full max-w-[520px] flex-col overflow-hidden rounded-xl border border-surface-700/70 bg-surface-900 shadow-2xl shadow-black/40"
		>
			<div class="px-5 py-4">
				<h2
					id="admin-shipments-filters-title"
					class="text-lg font-semibold text-surface-50"
				>
					{$_("shipments.filters")}
				</h2>
			</div>

			<div class="border-b border-surface-700/70 px-5 pt-4">
				<div
					role="tablist"
					aria-label={$_("shipments.filters")}
					class="flex gap-2"
				>
					<button
						id="admin-shipments-tab-office"
						type="button"
						role="tab"
						aria-controls="admin-shipments-panel-office"
						aria-selected={activeTab === "office"}
						tabindex={activeTab === "office" ? 0 : -1}
						onclick={() => (activeTab = "office")}
						class={[
							"cursor-pointer rounded-t-lg border border-b-0 px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50",
							activeTab === "office"
								? "border-surface-600 bg-surface-800 text-surface-50"
								: "border-transparent bg-transparent text-surface-400 hover:text-surface-200",
						]}
					>
						{$_("shipments.col.office")}
					</button>
					<button
						id="admin-shipments-tab-status"
						type="button"
						role="tab"
						aria-controls="admin-shipments-panel-status"
						aria-selected={activeTab === "status"}
						tabindex={activeTab === "status" ? 0 : -1}
						onclick={() => (activeTab = "status")}
						class={[
							"cursor-pointer rounded-t-lg border border-b-0 px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50",
							activeTab === "status"
								? "border-surface-600 bg-surface-800 text-surface-50"
								: "border-transparent bg-transparent text-surface-400 hover:text-surface-200",
						]}
					>
						{$_("shipments.col.status")}
					</button>
				</div>
			</div>

			<div class="min-h-0 flex-1 overflow-y-auto px-5 py-4">
				{#if activeTab === "office"}
					<div
						id="admin-shipments-panel-office"
						role="tabpanel"
						aria-labelledby="admin-shipments-tab-office"
						class="space-y-2"
					>
						<label
							class="flex cursor-pointer items-center gap-3 rounded-lg border border-surface-700 px-3 py-2 text-sm text-surface-200 transition-colors hover:bg-surface-800/60"
						>
							<input
								type="radio"
								name="office-filter"
								checked={draftOffice === null}
								onchange={() => setDraftOffice(null)}
								class="peer sr-only"
							/>
							<span
								class="relative flex h-5 w-5 items-center justify-center rounded-full border border-surface-600 bg-surface-950/70 transition-colors after:h-2.5 after:w-2.5 after:rounded-full after:bg-emerald-400 after:opacity-0 after:content-[''] after:transition-opacity peer-checked:border-emerald-400/80 peer-checked:after:opacity-100 peer-focus-visible:ring-1 peer-focus-visible:ring-emerald-400/60"
							>
							</span>
							<span
								>{$_(
									"admin.shipments.filters.all_offices",
								)}</span
							>
						</label>
						{#each offices as office (office)}
							<label
								class="flex cursor-pointer items-center gap-3 rounded-lg border border-surface-700 px-3 py-2 text-sm text-surface-200 transition-colors hover:bg-surface-800/60"
							>
								<input
									type="radio"
									name="office-filter"
									checked={draftOffice === office}
									onchange={() => setDraftOffice(office)}
									class="peer sr-only"
								/>
								<span
									class="relative flex h-5 w-5 items-center justify-center rounded-full border border-surface-600 bg-surface-950/70 transition-colors after:h-2.5 after:w-2.5 after:rounded-full after:bg-emerald-400 after:opacity-0 after:content-[''] after:transition-opacity peer-checked:border-emerald-400/80 peer-checked:after:opacity-100 peer-focus-visible:ring-1 peer-focus-visible:ring-emerald-400/60"
								>
								</span>
								<span>{office}</span>
							</label>
						{/each}
					</div>
				{:else}
					<div
						id="admin-shipments-panel-status"
						role="tabpanel"
						aria-labelledby="admin-shipments-tab-status"
						class="space-y-2"
					>
						<button
							type="button"
							onclick={clearDraftStatuses}
							class="cursor-pointer rounded-lg border border-surface-700 px-3 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-800/60 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
						>
							{$_("admin.shipments.filters.all_statuses")}
						</button>
						{#each SHIPMENT_STATUSES as status (status)}
							<label
								class="flex cursor-pointer items-center gap-3 rounded-lg border border-surface-700 px-3 py-2 text-sm text-surface-200 transition-colors hover:bg-surface-800/60"
							>
								<input
									type="checkbox"
									checked={draftStatuses.includes(status)}
									onchange={() => toggleDraftStatus(status)}
									class="peer sr-only"
								/>
								<span
									class="relative flex h-5 w-5 items-center justify-center rounded-md border border-surface-600 bg-surface-950/70 transition-colors peer-checked:border-emerald-400/80 peer-checked:bg-emerald-500/15 peer-focus-visible:ring-1 peer-focus-visible:ring-emerald-400/60 peer-checked:[&_svg]:opacity-100"
								>
									<svg
										class="h-3.5 w-3.5 text-emerald-300 opacity-0 transition-opacity"
										viewBox="0 0 20 20"
										fill="none"
										aria-hidden="true"
									>
										<path
											d="M5 10.5l3 3 7-7"
											stroke="currentColor"
											stroke-width="2.1"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								</span>
								<span>{$_(statusLabelKey(status))}</span>
							</label>
						{/each}
					</div>
				{/if}
			</div>

			<div
				class="flex items-center justify-between border-t border-surface-700/70 bg-surface-900/95 px-5 py-4"
			>
				<div class="flex items-center gap-2">
					<button
						type="button"
						onclick={cancelFilters}
						class="cursor-pointer rounded-lg border border-surface-700 px-4 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-800 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
					>
						{$_("admin.shipments.new.cancel")}
					</button>
					<button
						type="button"
						onclick={resetDraftFilters}
						class="cursor-pointer rounded-lg border border-surface-700 px-4 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-800 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
					>
						{$_("admin.shipments.filters.reset")}
					</button>
				</div>
				<button
					type="button"
					onclick={applyFilters}
					class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
				>
					{$_("admin.shipments.filters.apply")}
				</button>
			</div>
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
