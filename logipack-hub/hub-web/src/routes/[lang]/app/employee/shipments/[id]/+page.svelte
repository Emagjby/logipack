<script lang="ts">
	import type { PageData } from "./$types";
	import { page } from "$app/state";
	import { enhance } from "$app/forms";
	import { _ } from "svelte-i18n";
	import {
		formatDateTime,
		shortenHash,
		type StrataPackage,
	} from "$lib/domain/strataPackage";
	import { normalizeShipmentStatus } from "$lib/domain/shipmentStatus";
	import ShipmentStatusBadge from "$lib/components/app/ShipmentStatusBadge.svelte";
	import StrataPackageViewer from "$lib/components/app/StrataPackageViewer.svelte";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";

	let { data, form }: { data: PageData; form: any } = $props();

	let lang = $derived(data.pathname.split("/")[1] || "en");
	let selectedPackage = $state<StrataPackage | null>(null);
	let submitting = $state(false);
	let selectedStatus = $derived(form?.values?.to_status ?? "");
	let isOfficeRequired = $derived(selectedStatus === "in_transit");
	let isOfficeDisabled = $derived(selectedStatus !== "in_transit");
	let lastKnownHistoryOfficeId = $derived.by(() => {
		if (data.result.state !== "ok") return "";
		for (let i = data.result.statusHistory.length - 1; i >= 0; i--) {
			const office = data.result.statusHistory[i]?.office_id?.trim();
			if (office) return office;
		}
		return "";
	});
	let officeInputValue = $derived.by(() => {
		const fromForm = form?.values?.to_office_id;
		if (typeof fromForm === "string" && fromForm.trim().length > 0) {
			return fromForm;
		}
		return lastKnownHistoryOfficeId;
	});

	const TRANSITIONS: Record<string, string[]> = {
		new: ["accepted", "cancelled"],
		accepted: ["pending", "cancelled"],
		pending: ["in_transit", "cancelled"],
		in_transit: ["delivered", "cancelled"],
		delivered: [],
		cancelled: [],
		unknown: [],
	};

	function compactId(value: string): string {
		return `${value.slice(0, 8)}...`;
	}

	function isLikelyId(value: string): boolean {
		return /^[a-f0-9-]{8,}$/i.test(value);
	}

	function formatEventType(type: string): string {
		// Handle PascalCase (e.g. "ShipmentCreated" → "Shipment created")
		const hasUnderscore = type.includes("_");
		const words = hasUnderscore
			? type.split("_")
			: type.replace(/([a-z])([A-Z])/g, "$1 $2").split(" ");

		return words
			.map((word, i) =>
				i === 0
					? word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()
					: word.toLowerCase(),
			)
			.join(" ");
	}
</script>

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
			{$_("shipment.detail.error.title")}
		</h2>
		{#if data.result.message}
			<p class="mt-2 font-mono text-xs text-surface-600">
				{$_(data.result.message)}
			</p>
		{/if}
		<a
			href={`/${lang}/app/employee/shipments/${page.params.id}`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("shipment.detail.error.retry")}
		</a>
	</div>
{:else if data.result.state === "not_found"}
	<!-- Not-found state -->
	<div class="stagger stagger-1 flex flex-col items-center py-20 text-center">
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
					d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m8.25 3v6.75m0 0l-3-3m3 3l3-3M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125Z"
				/>
			</svg>
		</div>
		<h2 class="mt-4 text-lg font-semibold text-surface-50">
			{$_("shipment.detail.not_found.title")}
		</h2>
		<p class="mt-1 max-w-sm text-sm text-surface-400">
			{$_("shipment.detail.not_found.description")}
		</p>
		<a
			href={`/${lang}/app/employee/shipments`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("shipment.detail.not_found.back")}
		</a>
	</div>
{:else}
	{@const shipment = data.result.shipment}
	{@const statusHistory = data.result.statusHistory}
	{@const packages = data.result.packages}
	{@const offices =
		(
			data.result as {
				offices?: { id: string; name?: string | null }[];
			}
		).offices ??
		(
			data as {
				offices?: { id: string; name?: string | null }[];
			}
		).offices ??
		[]}
	{@const officeLabelById = new Map(
		offices
			.filter((office) => Boolean(office?.id))
			.map((office) => [office.id, office.name ?? office.id]),
	)}
	{@const orderedStatusHistory = [...statusHistory].sort(
		(a, b) =>
			new Date(a.changed_at).getTime() - new Date(b.changed_at).getTime(),
	)}
	{@const currentOfficeForHeader =
		orderedStatusHistory[orderedStatusHistory.length - 1]?.office_id ??
		shipment.current_office_id}

	<!-- 1. Header row -->
	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("shipment.detail.title")}
				{shipment.id}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("shipment.detail.last_updated", {
					values: { time: formatDateTime(shipment.updated_at, lang) },
				})}
			</p>
		</div>
		<div class="flex items-center gap-2">
			<a
				href={`/${lang}/app/employee/shipments`}
				class="rounded-lg bg-surface-800 px-3 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-700 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				{$_("shipment.detail.back")}
			</a>
		</div>
	</section>

	<!-- 2. Core fields panel -->
	<div
		class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5"
	>
		<dl class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
			<!-- Shipment ID -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.id")}
				</dt>
				<dd class="mt-1 flex items-center gap-2 text-sm">
					<span class="font-mono text-accent">{shipment.id}</span>
					<CopyIconButton
						value={shipment.id}
						title={$_("shipment.detail.copy_id")}
						ariaLabel={$_("shipment.detail.copy_id")}
						class="text-accent"
					/>
				</dd>
			</div>

			<!-- Client ID -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.client_id")}
				</dt>
				<dd class="mt-1 font-mono text-sm text-surface-200">
					{shipment.client_id}
				</dd>
			</div>

			<!-- Status -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.status")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					<ShipmentStatusBadge status={shipment.current_status} />
				</dd>
			</div>

			<!-- Current Office -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.current_office_id")}
				</dt>
				<dd class="mt-1 font-mono text-sm text-surface-200">
					{currentOfficeForHeader ?? $_("common.none")}
				</dd>
			</div>

			<!-- Created -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.created")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					{formatDateTime(shipment.created_at, lang)}
				</dd>
			</div>

			<!-- Updated -->
			<div>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("shipment.meta.updated")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					{formatDateTime(shipment.updated_at, lang)}
				</dd>
			</div>
		</dl>
	</div>

	<!-- 3. Update Status form -->
	{@const normalizedStatus = normalizeShipmentStatus(shipment.current_status)}
	{@const availableStatuses = TRANSITIONS[normalizedStatus] ?? []}
	{@const isTerminal = availableStatuses.length === 0}

	<div
		class="stagger stagger-3 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5"
	>
		<h2 class="text-sm font-semibold text-surface-50">
			{$_("shipment.update.title")}
		</h2>

		{#if isTerminal}
			<p class="mt-3 text-sm text-surface-200">
				{$_("shipment.update.terminal_hint")}
			</p>
		{:else}
			{#if form?.changeStatusError}
				<div
					class="mt-3 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-2.5 text-sm text-red-400"
				>
					{$_(form.changeStatusError)}
				</div>
			{/if}

			<form
				method="POST"
				action="?/changeStatus"
				use:enhance={() => {
					submitting = true;
					return async ({ update }) => {
						submitting = false;
						await update();
					};
				}}
				class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2"
			>
				<!-- New Status -->
				<div>
					<label
						for="to_status"
						class="block text-[11px] font-medium uppercase tracking-wider text-surface-600"
					>
						{$_("shipment.update.new_status")}
					</label>
					<select
						id="to_status"
						name="to_status"
						required
						bind:value={selectedStatus}
						class="mt-1 w-full rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 outline-none transition-colors focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
					>
						<option value="" disabled>
							{$_("shipment.update.select_status")}
						</option>
						{#each availableStatuses as status (status)}
							<option value={status}>
								{$_(`shipment_status.${status}`)}
							</option>
						{/each}
					</select>
				</div>

				<!-- Office ID -->
				<div>
					<label
						for="to_office_id"
						class="block text-[11px] font-medium uppercase tracking-wider text-surface-600"
					>
						{$_("shipment.update.office")}
					</label>
					<input
						type="text"
						id="to_office_id"
						name="to_office_id"
						required={isOfficeRequired}
						disabled={isOfficeDisabled}
						value={officeInputValue}
						placeholder={$_("shipment.update.office_placeholder")}
						class="mt-1 w-full rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-600 outline-none transition-colors focus:border-accent/50 focus:ring-1 focus:ring-accent/30 disabled:cursor-not-allowed disabled:opacity-60"
					/>
					<p class="mt-1 text-[11px] text-surface-600">
						{$_("shipment.update.office_hint")}
						{#if isOfficeDisabled}
							<span class="ml-1 text-surface-500">(In transit only)</span>
						{/if}
					</p>
				</div>

				<!-- Notes -->
				<div class="sm:col-span-2">
					<label
						for="notes"
						class="block text-[11px] font-medium uppercase tracking-wider text-surface-600"
					>
						{$_("shipment.update.notes")}
					</label>
					<textarea
						id="notes"
						name="notes"
						rows="2"
						placeholder={$_("shipment.update.notes_placeholder")}
						class="mt-1 w-full rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-600 outline-none transition-colors focus:border-accent/50 focus:ring-1 focus:ring-accent/30"
						>{form?.values?.notes ?? ""}</textarea
					>
				</div>

				<!-- Submit -->
				<div class="sm:col-span-2">
					<button
						type="submit"
						disabled={submitting}
						class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
					>
						{#if submitting}
							{$_("shipment.update.submitting")}
						{:else}
							{$_("shipment.update.submit")}
						{/if}
					</button>
				</div>
			</form>
		{/if}
	</div>

	<!-- 4. Status History panel -->
	<div
		class="stagger stagger-4 mt-6 rounded-xl border border-surface-700/50 bg-surface-900"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<h2 class="text-sm font-semibold text-surface-50">
				{$_("shipment.history.title")}
			</h2>
		</div>

		{#if orderedStatusHistory.length === 0}
			<div class="px-5 py-8 text-center text-sm text-surface-600">
				{$_("shipment.history.no_notes")}
			</div>
		{:else}
			<div class="overflow-x-hidden">
				<table class="w-full table-fixed">
					<thead>
						<tr>
							<th
								class="w-[20%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.changed_at")}
							</th>
							<th
								class="w-[14%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.from_status")}
							</th>
							<th
								class="w-[14%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.to_status")}
							</th>
							<th
								class="w-[20%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.office")}
							</th>
							<th
								class="w-[20%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.actor")}
							</th>
							<th
								class="w-[12%] px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.history.notes")}
							</th>
						</tr>
					</thead>
					<tbody>
						{#each orderedStatusHistory as row (row.id)}
							{@const officeLabel = row.office_id
								? (officeLabelById.get(row.office_id) ??
									row.office_id)
								: null}
							{@const officeValue = officeLabel ?? null}
							{@const officeHasRawId = Boolean(
								row.office_id && officeValue === row.office_id,
							)}
							{@const officeIsId = officeValue
								? isLikelyId(officeValue)
								: false}
							{@const actorValue = row.actor_user_id ?? null}
							{@const actorIsId = actorValue
								? isLikelyId(actorValue)
								: false}
							<tr
								class="border-t border-surface-800 transition-colors hover:bg-surface-800/50"
							>
								<td
									class="whitespace-nowrap px-5 py-3 text-xs text-surface-400"
								>
									{formatDateTime(row.changed_at, lang)}
								</td>
								<td class="px-5 py-3">
									{#if row.from_status}
										<ShipmentStatusBadge
											status={row.from_status}
											compact
										/>
									{:else}
										<span
											class="text-xs italic text-surface-600"
											>{$_("common.none")}</span
										>
									{/if}
								</td>
								<td class="px-5 py-3">
									<ShipmentStatusBadge
										status={row.to_status ?? "unknown"}
										compact
									/>
								</td>
								<td
									class="min-w-0 px-5 py-3 font-mono text-xs text-surface-400"
								>
									{#if officeValue}
										<div class="flex min-w-0 items-center gap-1.5">
											<span class="truncate">
												{officeHasRawId && officeIsId
													? compactId(officeValue)
													: officeValue}
											</span>
											{#if officeHasRawId && officeIsId}
												<CopyIconButton
													value={officeValue}
													title={$_(
														"shipment.detail.copy_id",
													)}
													ariaLabel={$_(
														"shipment.detail.copy_id",
													)}
												/>
											{/if}
										</div>
									{:else}
										{$_("common.none")}
									{/if}
								</td>
								<td
									class="min-w-0 px-5 py-3 font-mono text-xs text-surface-400"
								>
									{#if actorValue}
										<div class="flex min-w-0 items-center gap-1.5">
											<span class="truncate"
												>{actorIsId
													? compactId(actorValue)
													: actorValue}</span
											>
											{#if actorIsId}
												<CopyIconButton
													value={actorValue}
													title={$_(
														"shipment.detail.copy_id",
													)}
													ariaLabel={$_(
														"shipment.detail.copy_id",
													)}
												/>
											{/if}
										</div>
									{:else}
										{$_("common.none")}
									{/if}
								</td>
								<td
									class="min-w-0 max-w-[220px] px-5 py-3 text-xs text-surface-400"
								>
									{#if row.notes}
										<div class="group relative w-full">
											<p class="truncate">{row.notes}</p>
											<div
												class="pointer-events-none invisible absolute right-0 top-full z-10 mt-1 w-80 max-w-[min(24rem,75vw)] rounded-md border border-surface-700 bg-surface-900 px-2 py-1.5 text-xs text-surface-200 opacity-0 shadow-lg transition group-hover:visible group-hover:opacity-100"
											>
												{row.notes}
											</div>
										</div>
									{:else}
										{$_("shipment.history.no_notes")}
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>

	<!-- 5. Strata Timeline panel -->
	<div
		class="stagger stagger-5 mt-6 rounded-xl border border-surface-700/50 bg-surface-900"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<h2 class="text-sm font-semibold text-surface-50">
				{$_("shipment.strata.title")}
			</h2>
		</div>

		{#if packages.length === 0}
			<div class="px-5 py-8 text-center text-sm text-surface-600">
				{$_("common.none")}
			</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead>
						<tr>
							<th
								class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.strata.seq")}
							</th>
							<th
								class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.strata.event_type")}
							</th>
							<th
								class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.strata.hash")}
							</th>
							<th
								class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("shipment.strata.prev_hash")}
							</th>
						</tr>
					</thead>
					<tbody>
						{#each packages as pkg (pkg.hash)}
							<tr
								class="cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50"
								onclick={() => {
									selectedPackage = pkg;
								}}
								tabindex="0"
								role="link"
								onkeydown={(e) => {
									if (e.key === "Enter" || e.key === " ") {
										e.preventDefault();
										selectedPackage = pkg;
									}
								}}
							>
								<td
									class="px-5 py-3 font-mono text-xs text-surface-200"
								>
									#{pkg.seq}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{formatEventType(pkg.event_type)}
								</td>
								<td
									class="px-5 py-3 font-mono text-xs text-accent"
									title={pkg.hash}
								>
									{shortenHash(pkg.hash)}
								</td>
								<td
									class={[
										"px-5 py-3 font-mono text-xs",
										pkg.prev_hash
											? "text-surface-400"
											: "text-surface-600 italic",
									]}
									title={pkg.prev_hash ?? undefined}
								>
									{pkg.prev_hash
										? shortenHash(pkg.prev_hash)
										: $_("common.none")}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>

	<!-- 6. Strata Package Viewer modal -->
	<StrataPackageViewer
		pkg={selectedPackage}
		{lang}
		onclose={() => {
			selectedPackage = null;
		}}
	/>
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
	.stagger-3 {
		animation-delay: 0.15s;
	}
	.stagger-4 {
		animation-delay: 0.2s;
	}
	.stagger-5 {
		animation-delay: 0.25s;
	}
</style>
