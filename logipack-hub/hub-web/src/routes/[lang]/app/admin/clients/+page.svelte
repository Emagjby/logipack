<script lang="ts">
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();

	let lang = $derived(page.params.lang || "en");
	let clients = $derived(data.clients);
	let copiedClientId = $state<string | null>(null);
	let copiedClientIdTimer = $state<ReturnType<typeof setTimeout> | null>(
		null,
	);

	function openClient(id: string): void {
		void goto(`/${lang}/app/admin/clients/${id}`);
	}

	function openNewClient(): void {
		void goto(`/${lang}/app/admin/clients/new`);
	}

	function handleRowKeydown(event: KeyboardEvent, clientId: string): void {
		if (event.key === "Enter" || event.key === " ") {
			event.preventDefault();
			openClient(clientId);
		}
	}

	function compactClientId(id: string): string {
		const [head] = id.split("-");
		return `${head || id}...`;
	}

	async function copyClientId(
		event: MouseEvent,
		clientId: string,
	): Promise<void> {
		event.preventDefault();
		event.stopPropagation();

		try {
			await navigator.clipboard.writeText(clientId);
			copiedClientId = clientId;
			if (copiedClientIdTimer) clearTimeout(copiedClientIdTimer);
			copiedClientIdTimer = setTimeout(() => {
				if (copiedClientId === clientId) copiedClientId = null;
				copiedClientIdTimer = null;
			}, 1200);
		} catch {
			// Ignore clipboard failures (permission/browser restrictions).
		}
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
			{$_("admin.clients.error.headline")}
		</h2>
		<a
			href={`/${lang}/app/admin/clients`}
			class="mt-5 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.clients.retry")}
		</a>
	</div>
{:else}
	<section
		class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
	>
		<div>
			<h1 class="text-2xl font-bold text-surface-50">
				{$_("admin.clients.headline")}
			</h1>
			<p class="mt-1 text-sm text-surface-400">
				{$_("admin.clients.subtitle")}
			</p>
			<div class="mt-2">
				<span
					class="rounded-full border border-surface-700/50 bg-surface-900 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("admin.clients.scope", {
						values: { count: clients.length },
					})}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={openNewClient}
				class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.clients.new_client")}
			</button>
		</div>
	</section>

	{#if clients.length === 0}
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
				{$_("admin.clients.empty.headline")}
			</h2>
			<p class="mt-1 max-w-sm text-sm text-surface-400">
				{$_("admin.clients.empty.hint")}
			</p>
			<button
				type="button"
				onclick={openNewClient}
				class="mt-5 cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
			>
				{$_("admin.clients.new_client")}
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
								{$_("admin.clients.col.id")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.clients.col.name")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.clients.col.email")}
							</th>
							<th
								class="px-5 py-3 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
							>
								{$_("admin.clients.col.phone")}
							</th>
							<th class="w-10 px-5 py-3"></th>
						</tr>
					</thead>
					<tbody>
						{#each clients as client (client.id)}
							<tr
								onclick={() => openClient(client.id)}
								onkeydown={(event) =>
									handleRowKeydown(event, client.id)}
								class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50 focus-visible:bg-surface-800/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent/60"
								tabindex="0"
							>
								<td class="px-5 py-3 text-sm text-accent">
									<div class="flex items-center gap-2">
										<span class="font-mono"
											>{compactClientId(client.id)}</span
										>
										<button
											type="button"
											onclick={(event) =>
												copyClientId(event, client.id)}
											onkeydown={(event) =>
												event.stopPropagation()}
											class="rounded-md bg-surface-800 px-1.5 py-1 text-[11px] font-medium text-surface-400 transition-colors hover:bg-surface-700"
											title={$_("admin.clients.copy_id")}
											aria-label={$_(
												"admin.clients.copy_id",
											)}
										>
											{#if copiedClientId === client.id}
												{$_("admin.clients.copied")}
											{:else}
												<svg
													class="h-3.5 w-3.5"
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
									</div>
								</td>
								<td
									class="px-5 py-3 text-sm font-medium text-surface-50"
								>
									{client.name}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{#if client.email}
										{client.email}
									{:else}
										<span class="text-surface-600">—</span>
									{/if}
								</td>
								<td class="px-5 py-3 text-sm text-surface-200">
									{#if client.phone}
										{client.phone}
									{:else}
										<span class="text-surface-600">—</span>
									{/if}
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
