<script lang="ts">
	import { page } from "$app/state";
	import {
		normalizeShipmentStatus,
		statusLabelKey,
	} from "$lib/domain/shipmentStatus";
	import type { ReportName } from "$lib/server/hubApi";
	import { compactId, isIdColumn } from "$lib/utils/idDisplay";
	import { _ } from "svelte-i18n";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

	let lang = $derived(page.params.lang || "en");
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

	const reportOptions: { value: ReportName; labelKey: string }[] = [
		{
			value: "shipments-by-status",
			labelKey: "reports.option.shipments_by_status",
		},
		{
			value: "shipments-by-office",
			labelKey: "reports.option.shipments_by_office",
		},
		{
			value: "shipments-by-client",
			labelKey: "reports.option.shipments_by_client",
		},
		{
			value: "shipments-by-period",
			labelKey: "reports.option.shipments_by_period",
		},
	];

	const bucketOptions = [
		{ value: "day", labelKey: "reports.bucket.day" },
		{ value: "week", labelKey: "reports.bucket.week" },
		{ value: "month", labelKey: "reports.bucket.month" },
	];

	const reportColumnLabelKeys: Record<string, string> = {
		status: "reports.column.status",
		shipment_count: "reports.column.shipment_count",
		office_id: "reports.column.office_id",
		office_name: "reports.column.office_name",
		client_id: "reports.column.client_id",
		client_name: "reports.column.client_name",
		bucket_start: "reports.column.bucket_start",
	};

	function csvCell(value: string | number | boolean | null): string {
		const normalized = value === null ? "" : String(value);
		if (/[",\n]/.test(normalized)) {
			return `"${normalized.replaceAll('"', '""')}"`;
		}
		return normalized;
	}

	function downloadCsv() {
		if (data.result.state !== "ok") return;

		const lines = [
			data.result.report.columns.map(csvCell).join(","),
			...data.result.report.rows.map((row) => row.map(csvCell).join(",")),
		];
		const blob = new Blob([lines.join("\n")], {
			type: "text/csv;charset=utf-8",
		});
		const href = URL.createObjectURL(blob);
		const anchor = document.createElement("a");
		anchor.href = href;
		anchor.download = `${data.result.report.report_name}.csv`;
		anchor.click();
		URL.revokeObjectURL(href);
	}

	function formatGeneratedAt(iso: string): string {
		const timestamp = new Date(iso);
		if (Number.isNaN(timestamp.getTime())) return iso;
		return dateTimeFormat.format(timestamp);
	}

	function columnLabel(column: string): string {
		const key = reportColumnLabelKeys[column];
		if (key) {
			return $_(key);
		}

		return column
			.split("_")
			.map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
			.join(" ");
	}

	function isNumericColumn(column: string): boolean {
		return column.endsWith("_count");
	}

	function displayCell(column: string, cell: string | number | boolean | null): string {
		if (cell === null) return "—";

		if (typeof cell === "string") {
			if (column === "status") {
				return $_(statusLabelKey(normalizeShipmentStatus(cell)));
			}

			if (isIdColumn(column)) {
				return compactId(cell);
			}
		}

		return String(cell);
	}

	function cellTitle(column: string, cell: string | number | boolean | null): string | undefined {
		if (cell === null || typeof cell !== "string") return undefined;

		if (isIdColumn(column)) {
			return cell;
		}

		if (column === "status") {
			return $_(statusLabelKey(normalizeShipmentStatus(cell)));
		}

		return cell;
	}
</script>

<section class="flex flex-col gap-5">
	<div class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight text-surface-50">
				{$_("reports.headline")}
			</h1>
			<p class="mt-1 max-w-2xl text-sm leading-6 text-surface-400">
				{$_("reports.subtitle")}
			</p>
		</div>

		{#if data.result.state === "ok"}
			<button
				type="button"
				onclick={downloadCsv}
				class="inline-flex min-h-11 cursor-pointer items-center justify-center rounded-xl bg-accent px-5 py-2.5 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("reports.download_csv")}
			</button>
		{/if}
	</div>

	<form
		method="GET"
		class="rounded-2xl border border-surface-700/60 bg-surface-900/95 p-5 shadow-[0_24px_80px_rgba(0,0,0,0.18)]"
	>
		<div
			class={[
				"grid grid-cols-1 gap-3",
				data.selectedReport === "shipments-by-period"
					? "lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_minmax(0,0.9fr)_240px]"
					: "lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_240px]",
			]}
		>
			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-[11px] font-semibold uppercase tracking-[0.2em] text-surface-400"
					>{$_("reports.filter.from")}</span
				>
				<input
					type="date"
					name="from"
					value={data.from ?? ""}
					class="min-h-12 rounded-xl border border-surface-700 bg-surface-800/90 px-4 py-2.5 text-sm text-surface-50 transition-colors focus:border-accent/60 focus:outline-none focus:ring-2 focus:ring-accent/20"
				/>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-[11px] font-semibold uppercase tracking-[0.2em] text-surface-400"
					>{$_("reports.filter.to")}</span
				>
				<input
					type="date"
					name="to"
					value={data.to ?? ""}
					class="min-h-12 rounded-xl border border-surface-700 bg-surface-800/90 px-4 py-2.5 text-sm text-surface-50 transition-colors focus:border-accent/60 focus:outline-none focus:ring-2 focus:ring-accent/20"
				/>
			</label>

			{#if data.selectedReport === "shipments-by-period"}
				<label class="flex flex-col gap-2 text-sm text-surface-200">
					<span class="text-[11px] font-semibold uppercase tracking-[0.2em] text-surface-400"
						>{$_("reports.filter.bucket")}</span
					>
					<select
						name="bucket"
						value={data.selectedBucket}
						class="min-h-12 rounded-xl border border-surface-700 bg-surface-800/90 px-4 py-2.5 text-sm text-surface-50 transition-colors focus:border-accent/60 focus:outline-none focus:ring-2 focus:ring-accent/20"
					>
						{#each bucketOptions as option (option.value)}
							<option value={option.value}>{$_(option.labelKey)}</option>
						{/each}
					</select>
				</label>
			{/if}

			<div class="flex items-end">
				<button
					type="submit"
					class="min-h-12 w-full cursor-pointer rounded-xl bg-accent px-5 py-2.5 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
				>
					{$_("reports.run")}
				</button>
			</div>
		</div>

		<div class="mt-5">
			<p class="mb-3 text-[11px] font-semibold uppercase tracking-[0.2em] text-surface-400">
				{$_("reports.filter.report")}
			</p>

			<div class="flex flex-wrap gap-2">
				{#each reportOptions as option (option.value)}
					<button
						type="submit"
						name="report"
						value={option.value}
						class={[
							"cursor-pointer rounded-full border px-4 py-2 text-sm font-medium transition-colors",
							data.selectedReport === option.value
								? "border-accent bg-accent text-surface-950 shadow-[0_0_0_1px_rgba(44,214,101,0.15)]"
								: "border-surface-700 bg-surface-800 text-surface-300 hover:border-surface-600 hover:bg-surface-700",
						]}
					>
						{$_(option.labelKey)}
					</button>
				{/each}
			</div>
		</div>
	</form>

	<section class="rounded-2xl border border-surface-700/60 bg-surface-900/95 shadow-[0_24px_80px_rgba(0,0,0,0.18)]">
		<div class="border-b border-surface-700/50 px-5 py-5">
			<div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
				<div>
					<p class="text-[11px] font-semibold uppercase tracking-[0.2em] text-surface-500">
						{$_("reports.result_label")}
					</p>
					<h2 class="mt-2 text-lg font-semibold text-surface-50">
						{$_(`reports.option.${data.selectedReport.replaceAll("-", "_")}`)}
					</h2>
				</div>

				{#if data.result.state === "ok"}
					<div class="flex flex-wrap gap-2 text-xs">
						<span
							class="rounded-full border border-surface-700 bg-surface-950/50 px-3 py-1.5 text-surface-300"
						>
							{$_("reports.generated_at")}: {formatGeneratedAt(data.result.report.generated_at)}
						</span>
						<span
							class="rounded-full border border-surface-700 bg-surface-950/50 px-3 py-1.5 text-surface-300"
						>
							{$_("reports.rows_count", {
								values: { count: data.result.report.rows.length },
							})}
						</span>
						{#if data.selectedReport === "shipments-by-period"}
							<span
								class="rounded-full border border-accent/25 bg-accent/10 px-3 py-1.5 text-accent"
							>
								{$_("reports.active_bucket", {
									values: { bucket: $_(`reports.bucket.${data.selectedBucket}`) },
								})}
							</span>
						{/if}
					</div>
				{/if}
			</div>
		</div>

		{#if data.result.state === "error"}
			<div class="px-5 py-8 text-sm text-rose-300">{$_(data.result.message)}</div>
		{:else if data.result.report.rows.length === 0}
			<div class="px-5 py-8 text-sm text-surface-500">{$_("reports.empty")}</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full min-w-[560px] table-fixed">
					<thead class="bg-surface-950/30">
						<tr>
							{#each data.result.report.columns as column (column)}
								<th
									class={[
										"px-5 py-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-surface-500",
										isNumericColumn(column) ? "text-right" : "text-left",
									]}
								>
									{columnLabel(column)}
								</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each data.result.report.rows as row, rowIndex (`${rowIndex}`)}
							<tr class="border-t border-surface-800 transition-colors hover:bg-surface-950/20">
								{#each row as cell, cellIndex (`${rowIndex}-${cellIndex}`)}
									<td
										class={[
											"px-5 py-3 text-sm text-surface-200",
											isNumericColumn(data.result.report.columns[cellIndex] ?? "")
												? "text-right tabular-nums"
												: "text-left",
										]}
										title={cellTitle(
											data.result.report.columns[cellIndex] ?? "",
											cell,
										)}
									>
										<div class="min-w-0 truncate">
											{displayCell(data.result.report.columns[cellIndex] ?? "", cell)}
										</div>
									</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</section>
