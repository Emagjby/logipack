<script lang="ts">
	import type { ActionData } from "./$types";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import type { EmployeeDetail } from "$lib/server/hubApi/mappers/employees";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";

	type EmployeeDetailResult =
		| ({ state: "ok" } & EmployeeDetail)
		| { state: "not_found" }
		| { state: "error"; message: string };

	type EmployeeDetailPageData = {
		result: EmployeeDetailResult;
	};

	let {
		data,
		form,
	}: {
		data: EmployeeDetailPageData;
		form: ActionData | null;
	} = $props();

	let lang = $derived(page.params.lang || "en");
	let employeeId = $derived(page.params.id || "");
	let submitError = $derived(form?.submitError ?? null);

	function confirmDelete(event: SubmitEvent): void {
		if (!confirm($_("admin.employees.detail.delete_confirm"))) {
			event.preventDefault();
		}
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
			{$_("admin.employees.detail.error.headline")}
		</h2>
		{#if data.result.message}
			<p class="mt-2 font-mono text-xs text-surface-600">
				{$_(data.result.message)}
			</p>
		{/if}
		<a
			href={`/${lang}/app/admin/employees/${employeeId}`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.employees.detail.retry")}
		</a>
	</div>
{:else if data.result.state === "not_found"}
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
			{$_("admin.employees.detail.not_found")}
		</h2>
		<p class="mt-1 max-w-sm text-sm text-surface-400">
			{$_("admin.employees.detail.not_found_hint")}
		</p>
		<a
			href={`/${lang}/app/admin/employees`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.employees.detail.back_to_list")}
		</a>
	</div>
{:else}
	{@const employee = data.result}
	{@const officeName = employee.office_name ?? ""}
	{@const officeCity = employee.office_city ?? ""}
	{@const hasOffice = Boolean(officeName || officeCity)}
	{@const userName = employee.user_display_name ?? employee.full_name ?? employee.user_id}
	{@const userEmail = employee.email ?? "—"}

	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.employees.detail.headline", {
					values: { name: userName },
				})}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.employees.detail.subheadline", {
					values: { id: employee.id },
				})}
			</p>
		</div>
		<div class="flex flex-wrap items-start gap-2">
			<a
				href={`/${lang}/app/admin/employees`}
				class="rounded-lg bg-surface-800 px-3 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-700 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				{$_("admin.employees.detail.back_to_list")}
			</a>
			<a
				href={`/${lang}/app/admin/employees/${employee.id}/offices`}
				class="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				{hasOffice
					? $_("admin.employees.detail.change_office")
					: $_("admin.employees.detail.set_office")}
			</a>
			<div class="flex flex-col items-start gap-1 sm:items-end">
				<form method="POST" action="?/delete" onsubmit={confirmDelete}>
					<button
						type="submit"
						class="inline-flex cursor-pointer items-center justify-center rounded-lg bg-red-500 px-3 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-red-400 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-red-400/60"
					>
						{$_("admin.employees.detail.delete")}
					</button>
				</form>
			</div>
		</div>
	</section>

	{#if submitError}
		<div
			class="stagger stagger-2 mt-4 rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-300"
			aria-live="polite"
		>
			{$_(submitError)}
		</div>
	{/if}

	<div
		class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-4"
	>
		<dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
			<div class="rounded-lg bg-surface-900/40 p-2.5">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.user_name")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					{userName}
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.user_email")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					{userEmail}
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.user_id")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					<span class="font-mono text-accent">{employee.user_id}</span
					>
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.employee_id")}
				</dt>
				<dd class="mt-1 flex items-center gap-2 text-sm">
					<span class="font-mono text-accent">{employee.id}</span>
					<CopyIconButton
						value={employee.id}
						title={$_("admin.offices.copy_id")}
						ariaLabel={$_("admin.offices.copy_id")}
						class="text-accent"
					/>
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5 sm:col-span-2">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.office")}
				</dt>
			{#if hasOffice}
				<dd class="mt-1 text-sm text-surface-200">
					{officeName}
					{#if officeCity}
						<span class="text-surface-500">({officeCity})</span>
					{/if}
				</dd>
			{:else}
				<dd class="mt-1 text-sm text-surface-200">
					{$_("admin.employees.detail.no_office_assigned")}
				</dd>
			{/if}
	</div>
		</dl>
	</div>
{/if}
