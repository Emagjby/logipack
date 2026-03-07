<script lang="ts">
	import { enhance } from "$app/forms";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import type { ActionData, PageData } from "./$types";

	type EmployeeContext = {
		id: string;
		user_id: string;
		full_name: string;
		user_display_name: string | null;
	};

	type OfficeContext = {
		id: string;
		name: string;
		city: string;
		address: string;
	};

	type OfficesResult =
		| {
				state: "ok";
				employee: EmployeeContext;
				offices: OfficeContext[];
				currentOfficeId: string | null;
				currentOffice: OfficeContext | null;
				hasMultipleOffices: boolean;
		  }
		| { state: "error"; message?: string }
		| { state: "not_found" };

	type PageDataWithResult = PageData & { result: OfficesResult };

	type AssignOfficeActionData = ActionData & {
		fieldErrors?: { office_id?: string };
		submitError?: string | null;
		values?: { office_id?: string };
	};

	let {
		data,
		form,
	}: {
		data: PageDataWithResult;
		form: AssignOfficeActionData | null;
	} = $props();

	let lang = $derived(page.params.lang || "en");
	let employeeId = $derived(page.params.id || "");
	let submitting = $state(false);
	let selectedOfficeId = $derived(form?.values?.office_id ?? "");

	const enhanceSubmit = () => {
		submitting = true;
		return async ({ update }: { update: () => Promise<void> }) => {
			try {
				await update();
				window.location.reload();
			} finally {
				submitting = false;
			}
		};
	};

	let submitError = $derived(form?.submitError ?? null);
	let officeError = $derived(form?.fieldErrors?.office_id ?? null);

	function employeeNameLabel(employee: {
		user_display_name: string | null;
		full_name: string;
		user_id: string;
	}): string {
		return (
			employee.user_display_name ?? employee.full_name ?? employee.user_id
		);
	}

	function officeLabel(office: { name: string; city: string }): string {
		return `${office.name} (${office.city})`;
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
			{$_("admin.employees.offices.error.headline")}
		</h2>
		{#if data.result.message}
			<p class="mt-2 font-mono text-xs text-surface-600">
				{$_(data.result.message)}
			</p>
		{/if}
		<div class="mt-5 flex items-center gap-2">
			<a
				href={`/${lang}/app/admin/employees/${employeeId}/offices`}
				class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.employees.offices.error.retry")}
			</a>
			<a
				href={`/${lang}/app/admin/employees/${employeeId}`}
				class="rounded-lg border border-surface-700 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors hover:bg-surface-800"
			>
				{$_("admin.employees.offices.back_to_employee")}
			</a>
		</div>
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
	{@const offices = data.result.offices}
	{@const currentOffice = data.result.currentOffice}
	{@const hasCurrentOffice = Boolean(data.result.currentOfficeId)}

	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.employees.offices.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.employees.offices.employee_context", {
					values: { name: employeeNameLabel(employee) },
				})}
			</p>
		</div>
		<a
			href={`/${lang}/app/admin/employees/${employee.id}`}
			class="inline-flex cursor-pointer items-center justify-center rounded-lg border border-surface-700 bg-surface-800 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors hover:bg-surface-700 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
		>
			{$_("admin.employees.offices.back_to_employee")}
		</a>
	</section>

	<section
		class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-4"
	>
		<div class="rounded-lg bg-surface-900/40 p-2.5">
			<p
				class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
			>
				{$_("admin.employees.offices.current.headline")}
			</p>
			{#if currentOffice}
				<p class="mt-1 text-sm text-surface-200">
					{officeLabel(currentOffice)}
				</p>
				<p class="mt-1 text-xs text-surface-400">
					{currentOffice.address}
				</p>
			{:else}
				<p class="mt-1 text-sm text-surface-200">
					{$_("admin.employees.detail.no_office_assigned")}
				</p>
			{/if}
			{#if data.result.hasMultipleOffices}
				<p class="mt-2 text-xs text-amber-300">
					{$_("admin.employees.offices.current.multiple_warning")}
				</p>
			{/if}
		</div>
	</section>

	{#if offices.length === 0}
		<section
			class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-6 text-center"
		>
			<p class="text-sm text-surface-200">
				{$_("admin.employees.offices.empty.message")}
			</p>
			<a
				href={`/${lang}/app/admin/offices`}
				class="mt-5 inline-flex items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.employees.offices.empty.manage_offices")}
			</a>
		</section>
	{:else}
		<section
			class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5 sm:p-6"
		>
			<form
				method="POST"
				action="?/assign"
				class="space-y-5"
				use:enhance={enhanceSubmit}
			>
				{#if submitError}
					<div
						class="rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-300"
						aria-live="polite"
					>
						{$_(submitError)}
					</div>
				{/if}

				<div class="space-y-1.5">
					<label
						for="office_id"
						class="text-sm font-medium text-surface-200"
					>
						{$_("admin.employees.offices.form.office_id")}
						<span class="text-red-400">*</span>
					</label>
					<input
						id="office_id"
						name="office_id"
						type="text"
						value={selectedOfficeId}
						placeholder={$_(
							"admin.employees.offices.form.office_placeholder",
						)}
						required
						autocomplete="off"
						disabled={submitting}
						aria-invalid={officeError ? "true" : undefined}
						aria-describedby={officeError
							? "employee_office_hint employee_office_error"
							: "employee_office_hint"}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							officeError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					/>
					<p
						id="employee_office_hint"
						class="text-xs text-surface-400"
					>
						{$_("admin.employees.offices.form.office_hint")}
					</p>
					<p
						id="employee_office_error"
						class="min-h-4 text-xs text-red-400"
						aria-live="polite"
					>
						{officeError ? $_(officeError) : ""}
					</p>
				</div>

				<div
					class="mt-1 flex flex-col-reverse gap-2 pt-4 sm:flex-row sm:justify-end"
				>
					<button
						type="submit"
						disabled={submitting}
						class="inline-flex cursor-pointer items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:bg-surface-700 disabled:text-surface-400"
					>
						{submitting
							? $_("admin.employees.form.submitting")
							: hasCurrentOffice
								? $_("admin.employees.detail.change_office")
								: $_("admin.employees.detail.set_office")}
					</button>
				</div>
			</form>
		</section>
	{/if}
{/if}
