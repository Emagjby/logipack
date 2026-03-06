<script lang="ts">
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import type { PageData } from "./$types";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";

	let { data }: { data: PageData } = $props();

	let lang = $derived(page.params.lang || "en");
	let offices = $derived(data.offices);

	function openOffice(id: string): void {
		void goto(`/${lang}/app/admin/offices/${id}`);
	}

	function openNewOffice(): void {
		void goto(`/${lang}/app/admin/offices/new`);
	}

	function handleRowKeydown(event: KeyboardEvent, officeId: string): void {
		if (event.key === "Enter" || event.key === " ") {
			event.preventDefault();
			openOffice(officeId);
		}
	}

	function compactOfficeId(id: string): string {
		const [head] = id.split("-");
		return `${head || id}...`;
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
			{$_("admin.offices.error.headline")}
		</h2>
		<a
			href={`/${lang}/app/admin/offices`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.offices.retry")}
		</a>
	</div>
{:else}
	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.offices.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.offices.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("admin.offices.scope", { values: { count: offices.length } })}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={openNewOffice}
				class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.offices.new_office")}
			</button>
		</div>
	</section>

	{#if offices.length === 0}
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
				{$_("admin.offices.empty.headline")}
			</h2>
			<p class="mt-1 max-w-sm text-sm text-surface-400">
				{$_("admin.offices.empty.hint")}
			</p>
			<button
				type="button"
				onclick={openNewOffice}
				class="mt-5 cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.offices.new_office")}
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
								{$_("admin.offices.col.id")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.offices.col.name")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.offices.col.city")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.offices.col.address")}
							</th>
							<th class="w-10 px-5 py-3"></th>
						</tr>
					</thead>
					<tbody>
						{#each offices as office (office.id)}
							<tr
								onclick={() => openOffice(office.id)}
								onkeydown={(event) =>
									handleRowKeydown(event, office.id)}
								class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50 focus-visible:bg-surface-800/50 focus-visible:outline-none"
								tabindex="0"
							>
								<td class="px-5 py-3 text-sm text-accent">
									<div class="flex items-center gap-2">
										<span class="font-mono">{compactOfficeId(office.id)}</span>
										<CopyIconButton
											value={office.id}
											title={$_("admin.offices.copy_id")}
											ariaLabel={$_("admin.offices.copy_id")}
											stopPropagation
										/>
									</div>
								</td>
								<td class="px-5 py-3 text-sm font-medium text-surface-50">
									{office.name}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{office.city}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{office.address}
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
