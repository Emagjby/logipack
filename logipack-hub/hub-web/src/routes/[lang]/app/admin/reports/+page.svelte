<script lang="ts">
	import {
		normalizeShipmentStatus,
		statusLabelKey,
	} from "$lib/domain/shipmentStatus";
	import type { ReportName } from "$lib/server/hubApi";
	import { compactId, isIdColumn } from "$lib/utils/idDisplay";
	import { _ } from "svelte-i18n";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

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

	function formatDate(value: string | null): string {
		if (!value) return "";

		const dateOnlyMatch = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
		if (dateOnlyMatch) {
			return `${dateOnlyMatch[3]}/${dateOnlyMatch[2]}/${dateOnlyMatch[1]}`;
		}

		const timestamp = new Date(value);
		if (Number.isNaN(timestamp.getTime())) return value;

		const day = String(timestamp.getDate()).padStart(2, "0");
		const month = String(timestamp.getMonth() + 1).padStart(2, "0");
		const year = timestamp.getFullYear();
		return `${day}/${month}/${year}`;
	}

	function dateInputValue(value: string | null): string {
		if (!value) return "";

		const isoMatch = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
		if (isoMatch) return `${isoMatch[1]}-${isoMatch[2]}-${isoMatch[3]}`;

		const localMatch = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec(value);
		if (localMatch) return `${localMatch[3]}-${localMatch[2]}-${localMatch[1]}`;

		return "";
	}

	function isDateColumn(column: string): boolean {
		return column === "bucket_start";
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

			if (isDateColumn(column)) {
				return formatDate(cell);
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

		if (isDateColumn(column)) {
			return formatDate(cell);
		}

		return cell;
	}
</script>

<section class="flex flex-col gap-4">
	<div class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("reports.headline")}
			</h1>
			<p class="mt-1 max-w-2xl text-sm text-surface-400">
				{$_("reports.subtitle")}
			</p>
		</div>

		{#if data.result.state === "ok"}
			<button
				type="button"
				onclick={downloadCsv}
				class="inline-flex cursor-pointer items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("reports.download_csv")}
			</button>
		{/if}
	</div>

	<form
		method="GET"
		class="stagger stagger-2 rounded-xl border border-surface-700/50 bg-surface-900 p-5"
	>
		<input type="hidden" name="report" value={data.selectedReport} />

		<div
			class={[
				"grid grid-cols-1 gap-3",
				data.selectedReport === "shipments-by-period"
					? "lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_minmax(0,0.9fr)_240px]"
					: "lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_240px]",
			]}
		>
			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400"
					>{$_("reports.filter.from")}</span
				>
				<input
					type="date"
					name="from"
					value={dateInputValue(data.from)}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				/>
			</label>

			<label class="flex flex-col gap-2 text-sm text-surface-200">
				<span class="text-xs font-medium uppercase tracking-wider text-surface-400"
					>{$_("reports.filter.to")}</span
				>
				<input
					type="date"
					name="to"
					value={dateInputValue(data.to)}
					class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
				/>
			</label>

			{#if data.selectedReport === "shipments-by-period"}
				<label class="flex flex-col gap-2 text-sm text-surface-200">
					<span class="text-xs font-medium uppercase tracking-wider text-surface-400"
						>{$_("reports.filter.bucket")}</span
					>
					<select
						name="bucket"
						value={data.selectedBucket}
						class="rounded-lg border border-surface-700/50 bg-surface-800 px-3 py-2 text-sm text-surface-200 transition-colors focus:border-accent/50 focus:outline-none focus:ring-1 focus:ring-accent/50"
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
					class="w-full cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
				>
					{$_("reports.run")}
				</button>
			</div>
		</div>

		<div class="mt-5">
			<p class="mb-3 text-xs font-medium uppercase tracking-wider text-surface-400">
				{$_("reports.filter.report")}
			</p>

			<div class="flex flex-wrap gap-2">
				{#each reportOptions as option (option.value)}
					<button
						type="submit"
						name="report"
						value={option.value}
						class={[
							"cursor-pointer rounded-lg border px-3 py-1.5 text-xs font-semibold transition-colors",
							data.selectedReport === option.value
								? "border-accent bg-accent text-surface-950"
								: "border-surface-700 bg-surface-900 text-surface-100 hover:border-surface-500 hover:bg-surface-800",
						]}
					>
						{$_(option.labelKey)}
					</button>
				{/each}
			</div>
		</div>
	</form>

	<section class="stagger stagger-3 overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900">
		<div class="border-b border-surface-700/50 px-5 py-4">
			<div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
				<div>
					<p class="text-xs font-medium uppercase tracking-wider text-surface-400">
						{$_("reports.result_label")}
					</p>
					<h2 class="mt-1 text-sm font-semibold text-surface-50">
						{$_(`reports.option.${data.selectedReport.replaceAll("-", "_")}`)}
					</h2>
				</div>

				{#if data.result.state === "ok"}
					<div class="flex flex-wrap gap-2 text-xs">
						<span
							class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
						>
							{$_("reports.generated_at")}: {formatDate(data.result.report.generated_at)}
						</span>
						<span
							class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
						>
							{$_("reports.rows_count", {
								values: { count: data.result.report.rows.length },
							})}
						</span>
						{#if data.selectedReport === "shipments-by-period"}
							<span
								class="rounded-full bg-accent/10 px-2.5 py-1 text-xs font-semibold text-accent"
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
			<div class="flex flex-col items-center px-5 py-20 text-center text-sm text-rose-300">
				{$_(data.result.message)}
			</div>
		{:else if data.result.report.rows.length === 0}
			<div class="flex flex-col items-center px-5 py-20 text-center">
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-surface-800">
					<svg class="h-6 w-6 text-surface-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125C16.5 3.504 17.004 3 17.625 3h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
					</svg>
				</div>
				<p class="mt-4 text-sm font-medium text-surface-200">
					{$_("reports.empty")}
				</p>
			</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full min-w-[560px] table-fixed">
					<thead>
						<tr>
							{#each data.result.report.columns as column (column)}
								<th
									class={[
										"px-5 py-3 text-[11px] font-medium uppercase tracking-wider text-surface-600",
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
							<tr class="border-t border-surface-800 transition-colors hover:bg-surface-800/50">
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
