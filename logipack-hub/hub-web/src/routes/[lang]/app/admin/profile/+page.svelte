<script lang="ts">
	import { browser } from "$app/environment";
	import type { PageData } from "./$types";
	import { _ } from "svelte-i18n";
	import LanguageFlagDropdown from "$lib/components/app/LanguageFlagDropdown.svelte";
	import CopyIconButton from "$lib/components/app/CopyIconButton.svelte";

	let { data }: { data: PageData } = $props();

	let lang = $derived(data.pathname.split("/")[1] || "en");
	let notAssigned = $derived($_("profile.not_assigned"));

	function logout(): void {
		window.location.href = "/logout";
	}
</script>

<section
	class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
>
	<div>
		<h1 class="text-2xl font-bold text-surface-50">
			{$_("profile.title")}
		</h1>
		<p class="mt-1 text-sm text-surface-400">{$_("profile.subtitle")}</p>
	</div>

	{#if data.hasLogoutRoute}
		<div class="flex items-center gap-2 sm:pt-1">
			<button
				type="button"
				onclick={logout}
				class="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-surface-700 bg-surface-800/80 px-3.5 py-2 text-sm font-medium text-surface-400 transition-colors hover:bg-surface-700"
			>
				<svg
					class="h-4 w-4 shrink-0 text-surface-400"
					viewBox="0 0 20 20"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path
						d="M8 5H4.5A1.5 1.5 0 003 6.5v7A1.5 1.5 0 004.5 15H8"
					/>
					<path d="M12 13l3-3-3-3" />
					<path d="M15 10H7" />
				</svg>
				{$_("navbar.sign_out")}
			</button>
		</div>
	{/if}
</section>

<div class="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-[2fr_1fr]">
	<section class="rounded-xl border border-surface-700/50 bg-surface-900">
		<div class="border-b border-surface-700/50 px-6 py-4">
			<h2 class="text-sm font-semibold text-surface-50">
				{$_("profile.section.account")}
			</h2>
		</div>

		<dl class="divide-y divide-surface-800">
			<div
				class="grid gap-2 px-6 py-3 sm:grid-cols-[140px_minmax(0,1fr)_auto] sm:items-center"
			>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("profile.field.name")}
				</dt>
				<dd class="min-w-0 truncate text-sm text-surface-200">
					{data.profile.name ?? notAssigned}
				</dd>
				<CopyIconButton
					value={data.profile.name ?? ""}
					title={$_("profile.copy")}
					ariaLabel={`${$_("profile.copy")} ${$_("profile.field.name")}`}
					disabled={!browser || !data.profile.name}
					class="border border-surface-700"
				/>
			</div>

			<div
				class="grid gap-2 px-6 py-3 sm:grid-cols-[140px_minmax(0,1fr)_auto] sm:items-center"
			>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("profile.field.user_id")}
				</dt>
				<dd class="min-w-0 truncate font-mono text-sm text-surface-200">
					{data.profile.userId ?? notAssigned}
				</dd>
				<CopyIconButton
					value={data.profile.userId ?? ""}
					title={$_("profile.copy")}
					ariaLabel={`${$_("profile.copy")} ${$_("profile.field.user_id")}`}
					disabled={!browser || !data.profile.userId}
					class="border border-surface-700"
				/>
			</div>

			<div
				class="grid gap-2 px-6 py-3 sm:grid-cols-[140px_minmax(0,1fr)] sm:items-center"
			>
				<dt
					class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
				>
					{$_("profile.field.roles")}
				</dt>
				<dd>
					{#if data.profile.roles.length > 0}
						<div class="flex flex-wrap gap-1.5">
							{#each data.profile.roles as role (role)}
								<span
									class="rounded-full bg-accent/10 px-2 py-0.5 text-[11px] font-semibold text-accent"
								>
									{role}
								</span>
							{/each}
						</div>
					{:else}
						<span class="text-sm text-surface-200"
							>{notAssigned}</span
						>
					{/if}
				</dd>
			</div>
		</dl>
	</section>

	<section class="rounded-xl border border-surface-700/50 bg-surface-900">
		<div class="border-b border-surface-700/50 px-6 py-4">
			<h2 class="text-sm font-semibold text-surface-50">
				{$_("profile.section.preferences")}
			</h2>
		</div>
		<div class="space-y-2 px-6 py-4">
			<LanguageFlagDropdown
				pathname={data.pathname}
				{lang}
				align="left"
				fullWidth
				showCurrentLabel
			/>
			<p class="text-xs text-surface-600">
				{$_("profile.preferences.language_help")}
			</p>
		</div>
	</section>
</div>
