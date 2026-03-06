<script lang="ts">
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";

	type EmployeeListRow = {
		id: string;
		user_display_name: string | null;
		full_name: string | null;
		email: string;
		office_name: string | null;
		office_city: string | null;
	};

	type EmployeesPageData = {
		employees: EmployeeListRow[];
		loadError: boolean;
	};

	let { data }: { data: EmployeesPageData } = $props();

	let lang = $derived(page.params.lang || "en");
	let employees = $derived(data.employees);

	function compactId(value: string): string {
		return `${value.slice(0, 8)}...`;
	}

	function openEmployee(id: string): void {
		void goto(`/${lang}/app/admin/employees/${id}`);
	}

	function openNewEmployee(): void {
		void goto(`/${lang}/app/admin/employees/new`);
	}

	function handleRowKeydown(event: KeyboardEvent, employeeId: string): void {
		if (event.key === "Enter" || event.key === " ") {
			event.preventDefault();
			openEmployee(employeeId);
		}
	}

	function userNameLabel(employee: EmployeeListRow): string {
		return employee.user_display_name ?? employee.full_name ?? "";
	}

	function officeLabel(employee: EmployeeListRow): string | null {
		if (employee.office_name && employee.office_city) {
			return `${employee.office_name} (${employee.office_city})`;
		}
		if (employee.office_name) return employee.office_name;
		return null;
	}
</script>

{#if data.loadError}
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
			{$_("admin.employees.error.headline")}
		</h2>
		<p class="mt-1 max-w-sm text-sm text-surface-400">
			{$_("admin.employees.error.generic")}
		</p>
		<div class="mt-5 flex items-center gap-2">
			<a
				href={`/${lang}/app/admin/employees`}
				class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.employees.retry")}
			</a>
			<a
				href={`/${lang}/app/admin`}
				class="rounded-lg border border-surface-700 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors hover:bg-surface-800"
			>
				{$_("admin.employees.back")}
			</a>
		</div>
	</div>
{:else}
	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.employees.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.employees.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("admin.employees.scope", {
						values: { count: employees.length },
					})}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={openNewEmployee}
				class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.employees.new_employee")}
			</button>
		</div>
	</section>

	{#if employees.length === 0}
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
				{$_("admin.employees.empty.headline")}
			</h2>
			<button
				type="button"
				onclick={openNewEmployee}
				class="mt-5 cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.employees.new_employee")}
			</button>
		</div>
	{:else}
		<div
			class="stagger stagger-2 mt-4 overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900"
		>
			<div class="overflow-x-auto">
				<table class="w-full min-w-[680px]">
					<thead>
						<tr>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.employees.col.employee_id")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.employees.col.user")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.employees.col.email")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.employees.col.office")}
							</th>
							<th class="w-10 px-5 py-3"></th>
						</tr>
					</thead>
					<tbody>
						{#each employees as employee (employee.id)}
							<tr
								onclick={() => openEmployee(employee.id)}
								onkeydown={(event) =>
									handleRowKeydown(event, employee.id)}
								class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50 focus-visible:bg-surface-800/50 focus-visible:outline-none"
								tabindex="0"
								role="link"
							>
								<td class="px-5 py-3 text-sm text-accent">
									<div class="flex items-center gap-2">
										<span class="font-mono"
											>{compactId(employee.id)}</span
										>
									<CopyIconButton
										value={employee.id}
										title={$_("shipment.detail.copy_id")}
										ariaLabel={$_("shipment.detail.copy_id")}
										stopPropagation
									/>
									</div>
								</td>
								<td class="px-5 py-3 text-sm text-surface-50">
									{userNameLabel(employee)}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{employee.email}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{officeLabel(employee) ??
										$_("admin.employees.office.none")}
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
					</tbody>
				</table>
			</div>
		</div>
	{/if}
{/if}
