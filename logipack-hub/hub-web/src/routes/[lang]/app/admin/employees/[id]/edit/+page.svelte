<script lang="ts">
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import EmployeeForm from "$lib/components/app/EmployeeForm.svelte";
	import type { PageData } from "./$types";

	type NotFoundData = PageData & {
		notFound: true;
	};

	type FoundData = PageData & {
		notFound: false;
		employeeId: string;
		initialValues: {
			email: string;
		};
	};

	type EmployeeEditPageData = NotFoundData | FoundData;

	type EmployeeFormResult = {
		fieldErrors?: {
			email?: string;
		};
		values?: {
			email?: string;
		};
		submitError?: string;
	} | null;

	let {
		data,
		form,
	}: {
		data: EmployeeEditPageData;
		form: EmployeeFormResult;
	} = $props();
	let lang = $derived(page.params.lang || "en");
</script>

{#if data.notFound}
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
	{@const employeeData = data as FoundData}
	<section class="stagger stagger-1">
		<h1 class="text-2xl font-bold text-surface-50">
			{$_("admin.employees.edit.headline")}
		</h1>
		<p class="mt-1 max-w-2xl text-sm text-surface-400">
			{$_("admin.employees.edit.subtitle")}
		</p>
	</section>

	<EmployeeForm
		{form}
		initialValues={employeeData.initialValues}
		cancelHref={`/${lang}/app/admin/employees/${employeeData.employeeId}`}
		submitLabelKey="admin.employees.edit.submit"
	/>
{/if}
