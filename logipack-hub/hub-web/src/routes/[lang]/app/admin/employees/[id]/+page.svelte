<script lang="ts">
	/*
	import type { ActionData, PageData } from "./$types";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";

	let {
		data,
		form,
	}: {
		data: PageData;
		form: ActionData | null;
	} = $props();

	let lang = $derived(page.params.lang || "en");
	let employeeId = $derived(page.params.id || "");
	let copiedId = $state(false);
	let copyTimer = $state<ReturnType<typeof setTimeout> | null>(null);
	let submitError = $derived(form?.submitError ?? null);

	$effect(() => {
		return () => {
			if (copyTimer) {
				clearTimeout(copyTimer);
				copyTimer = null;
			}
		};
	});

	async function copyId(id: string): Promise<void> {
		try {
			await navigator.clipboard.writeText(id);
			copiedId = true;
			if (copyTimer) clearTimeout(copyTimer);
			copyTimer = setTimeout(() => {
				copiedId = false;
				copyTimer = null;
			}, 1200);
		} catch {
			// Ignore clipboard errors.
		}
	}

	function confirmDelete(event: SubmitEvent): void {
		if (!confirm($_("admin.employees.detail.delete_confirm"))) {
			event.preventDefault();
		}
	}
	*/
</script>

<!--
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
	{@const employee = data.result.employee}
	{@const user = data.result.user}
	{@const office = data.result.office}

	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.employees.detail.headline", {
					values: { name: user.name },
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
				{office
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
					{user.name}
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.user_email")}
				</dt>
				<dd class="mt-1 text-sm text-surface-200">
					{user.email}
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
					<button
						type="button"
						onclick={() => copyId(employee.id)}
						class="cursor-pointer rounded-md bg-surface-800 px-1.5 py-1 text-[0.62rem] font-medium text-accent transition-colors hover:bg-surface-700"
						title={$_("admin.offices.copy_id")}
						aria-label={$_("admin.offices.copy_id")}
					>
						{#if copiedId}
							{$_("admin.offices.copied")}
						{:else}
							<svg
								class="h-3.5 w-3.5 text-accent"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<rect
									x="9"
									y="9"
									width="11"
									height="11"
									rx="2"
								/>
								<path
									d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
								/>
							</svg>
						{/if}
					</button>
				</dd>
			</div>

			<div class="rounded-lg bg-surface-900/40 p-2.5 sm:col-span-2">
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("admin.employees.detail.office")}
				</dt>
				{#if office}
					<dd class="mt-1 text-sm text-surface-200">
						{office.name} ({office.city})
						<p class="mt-1 text-xs text-surface-400">
							{office.address}
						</p>
					</dd>
				{:else}
					<dd class="mt-1 text-sm text-surface-200">
						{$_("admin.employees.detail.no_office_assigned")}
					</dd>
				{/if}
				{#if data.result.hasMultipleOffices}
					<p class="mt-2 text-xs text-amber-300">
						{$_("admin.employees.detail.multi_office_warning")}
					</p>
				{/if}
			</div>
		</dl>
	</div>
{/if}
-->
