<script lang="ts">
	import { page } from "$app/state";
	import {
		normalizeShipmentStatus,
		statusLabelKey,
	} from "$lib/domain/shipmentStatus";
	import type { AuditEvent } from "$lib/server/hubApi";
	import { _ } from "svelte-i18n";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

	let lang = $derived(page.params.lang || "en");
	let events = $derived(data.result.state === "ok" ? data.result.events : []);
	let pageSize = $derived(data.result.limit);
	let firstPaginationPage = $derived(data.paginationPages[0] ?? data.currentPage);
	let lastPaginationPage = $derived(
		data.paginationPages[data.paginationPages.length - 1] ?? data.currentPage,
	);
	let dateTimeFormat = $derived.by(
		() =>
			new Intl.DateTimeFormat(lang, {
				month: "short",
				day: "numeric",
				year: "numeric",
				hour: "2-digit",
				minute: "2-digit",
				hour12: false,
			}),
	);

	const entityTypeOptions = ["shipment", "office", "client", "employee"];
	const actionOptions = [
		"shipment.created",
		"shipment.status_updated",
		"office.created",
		"office.updated",
		"office.deleted",
		"client.created",
		"client.updated",
		"client.deleted",
		"employee.created",
		"employee.updated",
		"employee.deleted",
		"employee.assigned_to_office",
		"employee.removed_from_office",
	];

	function formatTime(iso: string): string {
		const timestamp = new Date(iso);
		if (Number.isNaN(timestamp.getTime())) return "—";
		return dateTimeFormat.format(timestamp);
	}

	function truncateId(value: string): string {
		const normalized = value.trim();
		if (normalized.length <= 8) {
			return normalized;
		}
		return `${normalized.slice(0, 8)}...`;
	}

	function genericEntityKey(event: AuditEvent): string | null {
		switch (event.entity_type) {
			case "shipment":
				return "admin.audit.entity.shipment";
			case "office":
				return "admin.audit.entity.office";
			case "client":
				return "admin.audit.entity.client";
			case "employee":
				return "admin.audit.entity.employee";
			default:
				return null;
		}
	}

	function officeLabel(
		event: AuditEvent,
		idFormatter: (value: string) => string = truncateId,
	): string {
		if (event.office_label && !looksGenericEntityLabel("office", event.office_label)) {
			return event.office_label;
		}
		if (event.office_id) {
			return $_("admin.audit.entity.office", {
				values: { id: idFormatter(event.office_id) },
			});
		}
		return "—";
	}

	function actorFullLabel(event: AuditEvent): string {
		return (
			event.actor_display_name ??
			event.actor_user_id ??
			$_("admin.dashboard.actor.system")
		);
	}

	function actorLabel(event: AuditEvent): string {
		return event.actor_display_name ?? (event.actor_user_id ? truncateId(event.actor_user_id) : $_("admin.dashboard.actor.system"));
	}

	function metadataString(event: AuditEvent, key: string): string | null {
		const value = event.metadata?.[key];
		if (typeof value === "string" && value.trim()) {
			return value.trim();
		}
		if (typeof value === "number" || typeof value === "boolean") {
			return String(value);
		}
		return null;
	}

	function translatedStatus(value: string | null): string {
		if (!value) {
			return $_("shipment_status.unknown");
		}

		const normalized = normalizeShipmentStatus(value);
		return $_(statusLabelKey(normalized));
	}

	function looksGenericEntityLabel(
		entityType: AuditEvent["entity_type"],
		label: string,
	): boolean {
		const normalized = label.trim().toLowerCase();

		switch (entityType) {
			case "shipment":
				return normalized.startsWith("shipment ");
			case "office":
				return normalized.startsWith("office ");
			case "client":
				return normalized.startsWith("client ");
			case "employee":
				return normalized.startsWith("employee ");
			default:
				return false;
		}
	}

	function objectEntityLabel(
		event: AuditEvent,
		idFormatter: (value: string) => string = truncateId,
	): string {
		const key = genericEntityKey(event);

		if (event.entity_type === "shipment" && event.entity_id) {
			return $_("admin.audit.entity.shipment", {
				values: { id: idFormatter(event.entity_id) },
			});
		}

		if (event.entity_label && !looksGenericEntityLabel(event.entity_type, event.entity_label)) {
			return event.entity_label;
		}

		if (key && event.entity_id) {
			return $_(key, {
				values: { id: idFormatter(event.entity_id) },
			});
		}

		if (event.entity_type && event.entity_id) {
			return `${event.entity_type}:${idFormatter(event.entity_id)}`;
		}

		return "—";
	}

	function fullActionEntityValue(event: AuditEvent): string {
		if (event.entity_type === "shipment" && event.entity_id) {
			return event.entity_id;
		}

		if (event.entity_label && !looksGenericEntityLabel(event.entity_type, event.entity_label)) {
			return event.entity_label;
		}

		if (event.entity_id) {
			return event.entity_id;
		}

		return objectEntityLabel(event, (value) => value);
	}

	function actionEntityValue(
		event: AuditEvent,
		idFormatter: (value: string) => string = truncateId,
	): string {
		if (event.entity_type === "shipment" && event.entity_id) {
			return idFormatter(event.entity_id);
		}

		if (event.entity_label && !looksGenericEntityLabel(event.entity_type, event.entity_label)) {
			return event.entity_label;
		}

		if (event.entity_id) {
			return idFormatter(event.entity_id);
		}

		return objectEntityLabel(event, idFormatter);
	}

	function fallbackActionLabel(event: AuditEvent): string {
		const humanized = event.action_key
			.split(".")
			.map((part) => part.replaceAll("_", " "))
			.join(" ");
		return `${humanized}: ${actionEntityValue(event)}`;
	}

	function buildActionLabel(
		event: AuditEvent,
		idFormatter: (value: string) => string = truncateId,
	): string {
		const key = `admin.audit.action.${event.action_key}`;
		const translated = $_(key, {
			values: {
				entity: actionEntityValue(event, idFormatter),
				office: officeLabel(event, idFormatter),
				from_status: translatedStatus(metadataString(event, "from_status")),
				to_status: translatedStatus(metadataString(event, "to_status")),
			},
		});

		return translated === key ? fallbackActionLabel(event) : translated;
	}

	function actionLabel(event: AuditEvent): string {
		return buildActionLabel(event, truncateId);
	}

	function fullActionLabel(event: AuditEvent): string {
		return buildActionLabel(event, (value) => value);
	}

	function entityHref(event: AuditEvent): string | null {
		if (event.target_route) {
			return `/${lang}${event.target_route}`;
		}

		if (!event.entity_id) {
			return null;
		}

		switch (event.entity_type) {
			case "shipment":
				return `/${lang}/app/admin/shipments/${event.entity_id}`;
			case "office":
				return `/${lang}/app/admin/offices/${event.entity_id}`;
			case "client":
				return `/${lang}/app/admin/clients/${event.entity_id}`;
			case "employee":
				return `/${lang}/app/admin/employees/${event.entity_id}`;
			default:
				return null;
		}
	}

	function pageHref(pageNumber: number): string {
		const url = new URL(page.url);
		url.searchParams.delete("cursor");
		url.searchParams.delete("prev");
		url.searchParams.set("page", String(pageNumber));
		url.searchParams.set("limit", String(pageSize));

		return `${url.pathname}${url.search}`;
	}

	function hasFilters(): boolean {
		return Boolean(
			data.filters.actor ||
				data.filters.entityType ||
				data.filters.action ||
				data.filters.from ||
				data.filters.to,
		);
	}
</script>

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
			{$_("admin.audit.error.headline")}
		</h2>
		<a
			href={`/${lang}/app/admin/audit`}
			class="mt-5 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.audit.retry")}
		</a>
	</div>
{:else}
	<section class="stagger stagger-1">
		<h1 class="text-2xl font-bold text-surface-50">
			{$_("admin.audit.headline")}
		</h1>
		<p class="mt-1 text-sm text-surface-400">
			{$_("admin.audit.subtitle")}
		</p>
	</section>

	<form
		method="GET"
		class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5"
	>
		<input type="hidden" name="limit" value={pageSize} />

		<div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1.2fr)_minmax(0,0.9fr)_minmax(0,0.9fr)_auto]">
			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400">
					{$_("admin.audit.filters.actor")}
				</span>
				<input
					type="search"
					name="actor"
					value={data.filters.actor ?? ""}
					placeholder={$_("admin.audit.filters.actor_placeholder")}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors placeholder:text-surface-600 focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				/>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400">
					{$_("admin.audit.filters.type")}
				</span>
				<select
					name="entity_type"
					value={data.filters.entityType ?? ""}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				>
					<option value="">{$_("admin.audit.filters.any_type")}</option>
					{#each entityTypeOptions as entityType (entityType)}
						<option value={entityType}>
							{$_(`admin.audit.filters.type.${entityType}`)}
						</option>
					{/each}
				</select>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400">
					{$_("admin.audit.filters.action")}
				</span>
				<select
					name="action"
					value={data.filters.action ?? ""}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				>
					<option value="">{$_("admin.audit.filters.any_action")}</option>
					{#each actionOptions as action (action)}
						<option value={action}>
							{$_(`admin.audit.filters.action.${action}`)}
						</option>
					{/each}
				</select>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400">
					{$_("admin.audit.filters.from")}
				</span>
				<input
					type="date"
					name="from"
					value={data.filters.from ?? ""}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				/>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400">
					{$_("admin.audit.filters.to")}
				</span>
				<input
					type="date"
					name="to"
					value={data.filters.to ?? ""}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				/>
			</label>

			<div class="flex items-end gap-2 md:col-span-2 xl:col-span-1">
				<button
					type="submit"
					class="inline-flex flex-1 cursor-pointer items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover xl:flex-none"
				>
					{$_("admin.audit.filters.apply")}
				</button>
				{#if hasFilters()}
					<a
						href={`/${lang}/app/admin/audit?limit=${pageSize}`}
						class="inline-flex items-center justify-center rounded-lg border border-surface-700 bg-surface-900 px-3 py-2 text-sm font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
					>
						{$_("admin.audit.filters.clear")}
					</a>
				{/if}
			</div>
		</div>
	</form>

	{#if data.result.state === "empty"}
		<div
			class="stagger stagger-3 mt-6 flex flex-col items-center rounded-xl border border-surface-700/50 bg-surface-900 py-20 text-center"
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
			<p class="mt-4 text-sm text-surface-400">
				{$_("admin.audit.empty.headline")}
			</p>
		</div>
	{:else}
		<div
			class="stagger stagger-3 mt-4 overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900"
		>
			<div class="overflow-x-auto">
				<table class="w-full min-w-[720px]">
					<thead>
						<tr>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.audit.col.time")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.audit.col.actor")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.audit.col.action")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.audit.col.entity")}
							</th>
						</tr>
					</thead>
					<tbody>
						{#each events as event (event.id)}
							<tr class="border-t border-surface-800 transition-colors hover:bg-surface-800/50">
								<td class="px-5 py-3 align-top text-sm text-surface-400">
									{formatTime(event.occurred_at)}
								</td>
								<td
									class="px-5 py-3 align-top text-sm font-medium text-surface-50"
									title={actorFullLabel(event)}
								>
									{actorLabel(event)}
								</td>
								<td
									class="px-5 py-3 align-top text-sm leading-snug text-surface-200"
									title={fullActionLabel(event)}
								>
									{actionLabel(event)}
								</td>
								<td class="px-5 py-3 align-top text-sm leading-snug text-surface-200">
									{#if entityHref(event)}
										<a
											href={entityHref(event) ?? "#"}
											title={objectEntityLabel(event, (value) => value)}
											class="break-words text-accent transition-colors hover:text-accent-hover hover:underline"
										>
											{objectEntityLabel(event)}
										</a>
									{:else}
										<span title={objectEntityLabel(event, (value) => value)}>
											{objectEntityLabel(event)}
										</span>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div
				class="flex flex-col gap-3 border-t border-surface-800 px-5 py-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="text-xs text-surface-600">
					{$_("admin.audit.pagination.summary", {
						values: {
							count: events.length,
							total: data.result.totalCount ?? events.length,
						},
					})}
				</div>

				<nav
					class="flex items-center gap-2"
					aria-label={$_("admin.audit.pagination.label")}
				>
					{#if data.previousPageHref}
						<a
							href={data.previousPageHref}
							class="inline-flex items-center gap-1.5 rounded-lg border border-surface-700 bg-surface-900 px-3 py-1.5 text-xs font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
						>
							<svg
								class="h-3.5 w-3.5"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M15 19l-7-7 7-7"
								/>
							</svg>
							{$_("admin.audit.pagination.previous")}
						</a>
					{:else}
						<span
							class="inline-flex cursor-not-allowed items-center gap-1.5 rounded-lg border border-surface-800 bg-surface-900/60 px-3 py-1.5 text-xs font-semibold text-surface-700"
							aria-disabled="true"
						>
							<svg
								class="h-3.5 w-3.5"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M15 19l-7-7 7-7"
								/>
							</svg>
							{$_("admin.audit.pagination.previous")}
						</span>
					{/if}


					<div class="flex items-center gap-1" aria-hidden="false">
						{#if firstPaginationPage > 1}
							<a
								href={pageHref(1)}
								class="flex h-8 min-w-8 items-center justify-center rounded-lg border border-surface-700 bg-surface-900 px-2 text-xs font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
							>
								1
							</a>
							{#if firstPaginationPage > 2}
								<span class="px-1 text-xs font-semibold text-surface-600">...</span>
							{/if}
						{/if}

						{#each data.paginationPages as pageNumber (pageNumber)}
							{#if pageNumber === data.currentPage}
								<span
									class="flex h-8 min-w-8 items-center justify-center rounded-lg border border-accent bg-accent px-2 text-xs font-semibold text-surface-950"
									aria-current="page"
								>
									{pageNumber}
								</span>
							{:else}
								<a
									href={pageHref(pageNumber)}
									class="flex h-8 min-w-8 items-center justify-center rounded-lg border border-surface-700 bg-surface-900 px-2 text-xs font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
								>
									{pageNumber}
								</a>
							{/if}
						{/each}

						{#if data.totalPages && lastPaginationPage < data.totalPages}
							{#if lastPaginationPage < data.totalPages - 1}
								<span class="px-1 text-xs font-semibold text-surface-600">...</span>
							{/if}
							<a
								href={pageHref(data.totalPages)}
								class="flex h-8 min-w-8 items-center justify-center rounded-lg border border-surface-700 bg-surface-900 px-2 text-xs font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
							>
								{data.totalPages}
							</a>
						{/if}
					</div>

					{#if data.nextPageHref}
						<a
							href={data.nextPageHref}
							class="inline-flex items-center gap-1.5 rounded-lg border border-surface-700 bg-surface-900 px-3 py-1.5 text-xs font-semibold text-surface-100 transition-colors hover:border-surface-500 hover:bg-surface-800"
						>
							{$_("admin.audit.pagination.next")}
							<svg
								class="h-3.5 w-3.5"
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
						</a>
					{:else}
						<span
							class="inline-flex cursor-not-allowed items-center gap-1.5 rounded-lg border border-surface-800 bg-surface-900/60 px-3 py-1.5 text-xs font-semibold text-surface-700"
							aria-disabled="true"
						>
							{$_("admin.audit.pagination.next")}
							<svg
								class="h-3.5 w-3.5"
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
						</span>
					{/if}
				</nav>
			</div>
		</div>
	{/if}
{/if}
