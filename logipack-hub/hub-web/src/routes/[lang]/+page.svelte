<script lang="ts">
	import type { PageProps } from "./$types";
	import { ago } from "$lib/time";
	import { _, locale } from "svelte-i18n";
	import { page } from "$app/stores";
	import LanguageFlagDropdown from "$lib/components/app/LanguageFlagDropdown.svelte";

	let { data }: PageProps = $props();

	const currentLang = $derived($locale ?? $page.params.lang ?? "en");

	const principles = $derived([
		$_("principles.append_only_timeline"),
		$_("principles.role_based_access"),
		$_("principles.shipment_tracking"),
	]);

	const features = $derived([
		{
			title: $_("wyg.shipment_timeline.headline"),
			description: $_("wyg.shipment_timeline.description"),
			icon: "timeline",
		},
		{
			title: $_("wyg.office_aware_tracking.headline"),
			description: $_("wyg.office_aware_tracking.description"),
			icon: "office",
		},
		{
			title: $_("wyg.role_based_consoles.headline"),
			description: $_("wyg.role_based_consoles.description"),
			icon: "roles",
		},
		{
			title: $_("wyg.admin_management.headline"),
			description: $_("wyg.admin_management.description"),
			icon: "admin",
		},
	]);

	const steps = $derived([
		{
			number: 1,
			title: $_("hiw.first_step.headline"),
			description: $_("hiw.first_step.description"),
		},
		{
			number: 2,
			title: $_("hiw.second_step.headline"),
			description: $_("hiw.second_step.description"),
		},
		{
			number: 3,
			title: $_("hiw.third_step.headline"),
			description: $_("hiw.third_step.description"),
		},
	]);

	const stepPills: Record<number, { color: string }> = {
		1: { color: "bg-blue-500/10 text-blue-400" },
		2: { color: "bg-amber-500/10 text-amber-400" },
		3: { color: "bg-accent/10 text-accent" },
	};

	const stepLabels = $derived<Record<number, string>>({
		1: $_("hiw.first_step.label"),
		2: $_("hiw.second_step.label"),
		3: $_("hiw.third_step.label"),
	});
</script>

{#snippet featureIcon(name: string)}
	{#if name === "timeline"}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<circle cx="12" cy="12" r="10" />
			<polyline points="12 6 12 12 16 14" />
		</svg>
	{:else if name === "office"}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<rect x="4" y="2" width="16" height="20" rx="2" />
			<line x1="9" y1="6" x2="9" y2="6.01" />
			<line x1="15" y1="6" x2="15" y2="6.01" />
			<line x1="9" y1="10" x2="9" y2="10.01" />
			<line x1="15" y1="10" x2="15" y2="10.01" />
			<line x1="9" y1="14" x2="9" y2="14.01" />
			<line x1="15" y1="14" x2="15" y2="14.01" />
			<line x1="9" y1="18" x2="15" y2="18" />
		</svg>
	{:else if name === "roles"}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
			<path d="M12 8v4" />
			<circle cx="12" cy="16" r="0.5" />
		</svg>
	{:else if name === "admin"}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<circle cx="12" cy="12" r="3" />
			<path
				d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1.08-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1.08 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1.08z"
			/>
		</svg>
	{/if}
{/snippet}

<!-- Topbar -->
<header class="fixed top-4 left-1/2 -translate-x-1/2 w-[calc(100%-2rem)] z-50">
	<nav
		aria-label="Primary navigation"
		class="mx-auto flex md:max-w-6xl items-center justify-between rounded-xl border border-white/10 bg-surface-900/80 px-5 py-3 backdrop-blur-md"
	>
		<div class="flex items-center gap-3">
			<img
				src="https://raw.githubusercontent.com/Emagjby/logipack-assets/refs/heads/main/logipack-crate-green.png"
				alt="LogiPack Logo"
				class="h-6 w-6 rounded-sm object-cover"
			/>
			<span class="text-lg font-bold text-surface-50">LogiPack</span>
		</div>
		<div class="flex items-center gap-3">
			<LanguageFlagDropdown pathname={$page.url.pathname} lang={currentLang} />

			<a
				href={data.loginUrl}
				data-sveltekit-reload
				class="cursor-pointer rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors duration-200 hover:bg-accent-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-950"
			>
				{$_("hero.cta_primary")}
			</a>
		</div>
	</nav>
</header>

<main id="main" class="relative min-h-dvh overflow-x-clip bg-surface-950">
	<!-- Glow container -->
	<div class="pointer-events-none absolute inset-0 overflow-hidden">
		<div
			class="absolute -top-32 -left-32 h-125 w-125 rounded-full bg-accent/5 blur-3xl"
		></div>
		<div
			class="absolute -right-32 -bottom-32 h-125 w-125 rounded-full bg-blue-500/5 blur-3xl"
		></div>
	</div>

	<!-- Hero -->
	<section aria-label="Hero" class="relative overflow-hidden pt-28 pb-16">
		<div
			class="mx-auto max-w-6xl px-6 lg:grid lg:grid-cols-2 lg:items-center lg:gap-12"
		>
			<!-- Left column -->
			<div>
				<h1
					class="text-4xl font-bold leading-tight text-surface-50 md:text-5xl"
				>
					{$_("hero.headline")}
					<span
						class="decoration-accent/40 underline underline-offset-4 decoration-2"
						>{$_("hero.headline_highlighted")}</span
					>.
				</h1>
				<p class="mt-4 max-w-lg text-lg text-surface-400">
					{$_("hero.subheadline")}
				</p>
				<div class="mt-8 flex items-center gap-4">
					<a
						href={data.loginUrl}
						data-sveltekit-reload
						class="cursor-pointer rounded-lg bg-accent px-6 py-3 font-semibold text-surface-950 transition-colors duration-200 hover:bg-accent-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-950"
					>
						{$_("hero.cta_primary")}
					</a>
					<a
						href="#features"
						class="cursor-pointer text-surface-400 px-6 py-3 underline underline-offset-4 transition-colors duration-200 hover:text-surface-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-950"
					>
						{$_("hero.cta_secondary")}
					</a>
				</div>
			</div>

			<!-- Right column — Product Preview Mock -->
			<div class="relative mt-10 lg:mt-0" aria-hidden="true">
				<!-- Subtle glow behind the preview -->
				<div
					class="absolute -inset-4 rounded-2xl bg-accent/5 blur-2xl"
				></div>

				<!-- Preview frame -->
				<div
					class="relative overflow-hidden rounded-2xl border border-white/10 bg-surface-900/80 shadow-2xl shadow-black/20 backdrop-blur-sm"
				>
					<!-- Window chrome bar -->
					<div
						class="flex items-center gap-2 border-b border-white/5 bg-surface-900 px-4 py-2.5"
					>
						<div class="flex gap-1.5">
							<div
								class="h-2.5 w-2.5 rounded-full bg-surface-700"
							></div>
							<div
								class="h-2.5 w-2.5 rounded-full bg-surface-700"
							></div>
							<div
								class="h-2.5 w-2.5 rounded-full bg-surface-700"
							></div>
						</div>
						<div class="ml-3 flex-1">
							<div
								class="mx-auto max-w-50 rounded-md bg-surface-800 px-3 py-1 text-center text-xs text-surface-600"
							>
								logipack.com/app/shipments
							</div>
						</div>
					</div>

					<!-- Search bar -->
					<div class="border-b border-white/5 px-4 py-3">
						<div
							class="flex items-center gap-2 rounded-lg border border-white/10 bg-surface-950/50 px-3 py-2"
						>
							<svg
								class="h-4 w-4 text-surface-600"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								><circle cx="11" cy="11" r="8" /><line
									x1="21"
									y1="21"
									x2="16.65"
									y2="16.65"
								/></svg
							>
							<span class="text-xs text-surface-600"
								>{$_("hero.preview.search")}</span
							>
						</div>
					</div>

					<!-- Mini shipments table -->
					<div class="px-4 pt-3 pb-1">
						<table class="w-full text-xs">
							<thead>
								<tr class="text-left text-surface-600">
									<th class="pb-2 pr-3 font-medium"
										>{$_("hero.preview.th.id")}</th
									>
									<th class="pb-2 pr-3 font-medium"
										>{$_("hero.preview.th.status")}</th
									>
									<th
										class="hidden pb-2 pr-3 font-medium sm:table-cell"
										>{$_("hero.preview.th.office")}</th
									>
									<th class="pb-2 font-medium text-right"
										>{$_("hero.preview.th.updated")}</th
									>
								</tr>
							</thead>
							<tbody class="text-surface-400">
								<tr class="border-t border-white/5">
									<td
										class="py-2 pr-3 font-mono text-surface-50"
										>SHP-2617e317...</td
									>
									<td class="py-2 pr-3">
										<span
											class="inline-flex items-center rounded-full bg-amber-500/10 px-2 py-0.5 text-[11px] font-medium text-amber-400"
											>{$_(
												"hero.preview.status.in_transit",
											)}</span
										>
									</td>
									<td class="hidden py-2 pr-3 sm:table-cell"
										>{$_("hero.preview.office.sofia")}</td
									>
									<td class="py-2 text-right text-surface-600"
										>{$_(
											"hero.preview.updated.two_min_ago",
										)}</td
									>
								</tr>
								<tr class="border-t border-white/5">
									<td
										class="py-2 pr-3 font-mono text-surface-50"
										>SHP-67af1923...</td
									>
									<td class="py-2 pr-3">
										<span
											class="inline-flex items-center rounded-full bg-accent/10 px-2 py-0.5 text-[11px] font-medium text-accent"
											>{$_(
												"hero.preview.status.delivered",
											)}</span
										>
									</td>
									<td class="hidden py-2 pr-3 sm:table-cell"
										>{$_("hero.preview.office.plovdiv")}</td
									>
									<td class="py-2 text-right text-surface-600"
										>{$_(
											"hero.preview.updated.eighteen_min_ago",
										)}</td
									>
								</tr>
								<tr class="border-t border-white/5">
									<td
										class="py-2 pr-3 font-mono text-surface-50"
										>SHP-84e9d01d...</td
									>
									<td class="py-2 pr-3">
										<span
											class="inline-flex items-center rounded-full bg-blue-500/10 px-2 py-0.5 text-[11px] font-medium text-blue-400"
											>{$_(
												"hero.preview.status.processing",
											)}</span
										>
									</td>
									<td class="hidden py-2 pr-3 sm:table-cell"
										>{$_("hero.preview.office.varna")}</td
									>
									<td class="py-2 text-right text-surface-600"
										>{$_(
											"hero.preview.updated.one_hour_ago",
										)}</td
									>
								</tr>
								<tr class="border-t border-white/5">
									<td
										class="py-2 pr-3 font-mono text-surface-50"
										>SHP-48fa935d...</td
									>
									<td class="py-2 pr-3">
										<span
											class="inline-flex items-center rounded-full bg-red-500/10 px-2 py-0.5 text-[11px] font-medium text-red-400"
											>{$_(
												"hero.preview.status.cancelled",
											)}</span
										>
									</td>
									<td class="hidden py-2 pr-3 sm:table-cell"
										>{$_("hero.preview.office.sofia")}</td
									>
									<td class="py-2 text-right text-surface-600"
										>{$_(
											"hero.preview.updated.three_hours_ago",
										)}</td
									>
								</tr>
							</tbody>
						</table>
					</div>

					<!-- Mini timeline panel -->
					<div class="border-t border-white/5 px-4 py-3">
						<div class="mb-2 flex items-center gap-2">
							<span
								class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
								>{$_("hero.preview.label.timeline")}</span
							>
							<span
								class="rounded bg-surface-800 px-1.5 py-0.5 text-[10px] text-surface-600"
								>SHP-2617e317...</span
							>
						</div>
						<div class="space-y-2">
							<div class="flex items-start gap-2.5">
								<div
									class="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-accent"
								></div>
								<div
									class="flex flex-1 items-baseline justify-between gap-2"
								>
									<span class="text-xs text-surface-400"
										>{$_(
											"hero.preview.timeline.first_event.headline",
										)}
										<span class="text-surface-200"
											>{$_(
												"hero.preview.timeline.first_event.headline_highlighted",
											)}</span
										></span
									>
									<span
										class="shrink-0 text-[11px] text-surface-600"
										>{ago(2)}</span
									>
								</div>
							</div>
							<div class="flex items-start gap-2.5">
								<div
									class="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-surface-600"
								></div>
								<div
									class="flex flex-1 items-baseline justify-between gap-2"
								>
									<span class="text-xs text-surface-400"
										>{$_(
											"hero.preview.timeline.second_event.headline",
										)}<span class="text-surface-200"
											>{$_(
												"hero.preview.timeline.second_event.headline_highlighted",
											)}</span
										></span
									>
									<span
										class="shrink-0 text-[11px] text-surface-600"
										>{ago(201)}</span
									>
								</div>
							</div>
							<div class="flex items-start gap-2.5">
								<div
									class="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-surface-600"
								></div>
								<div
									class="flex flex-1 items-baseline justify-between gap-2"
								>
									<span class="text-xs text-surface-400"
										>{$_(
											"hero.preview.timeline.third_event.headline",
										)}<span class="text-surface-200"
											>{$_(
												"hero.preview.timeline.third_event.headline_highlighted",
											)}</span
										>{$_(
											"hero.preview.timeline.third_event.headline_continuation",
										)}</span
									>
									<span
										class="shrink-0 text-[11px] text-surface-600"
										>{ago(245)}</span
									>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Principles strip -->
	<div class="mx-auto max-w-6xl px-6 pb-12">
		<div class="flex flex-wrap items-center justify-center gap-3">
			{#each principles as principle (principle)}
				<span
					class="rounded-full border border-white/5 bg-surface-900/40 px-4 py-1.5 text-xs text-surface-400"
				>
					{principle}
				</span>
			{/each}
		</div>
	</div>

	<!-- Features -->
	<section
		id="features"
		aria-label="Features"
		class="mx-auto max-w-6xl px-6 pb-16"
	>
		<div class="mb-8">
			<h2 class="text-sm uppercase tracking-widest text-surface-400">
				{$_("wyg.headline")}
			</h2>
			<p class="mt-2 text-surface-600 text-sm">
				{$_("wyg.subheadline")}
			</p>
		</div>

		<!-- Featured card (Shipment timeline) — full width -->
		<div
			class="group relative mb-4 cursor-pointer overflow-hidden rounded-xl border border-white/10 bg-surface-900/50 p-6 transition-all duration-200 hover:-translate-y-0.5 hover:border-accent/20 hover:shadow-lg hover:shadow-accent/5 md:flex md:items-start md:gap-6 md:p-8"
		>
			<div
				class="mb-4 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10 text-accent md:mb-0"
			>
				{@render featureIcon("timeline")}
			</div>
			<div class="flex-1">
				<div class="flex flex-wrap items-center gap-2">
					<h3 class="text-lg font-semibold text-surface-50">
						{$_("wyg.shipment_timeline.headline")}
					</h3>
					<span
						class="rounded-full bg-accent/10 px-2.5 py-0.5 text-[11px] font-medium text-accent"
						>{$_("wyg.shipment_timeline.label")}</span
					>
				</div>
				<p class="mt-1.5 text-sm leading-relaxed text-surface-400">
					{$_("wyg.shipment_timeline.description")}
				</p>
			</div>
		</div>

		<!-- Remaining 3 cards — 3-column grid on md -->
		<div class="grid gap-4 sm:grid-cols-2 md:grid-cols-3">
			{#each features.slice(1) as feature (feature.icon)}
				<div
					class="group relative cursor-pointer overflow-hidden rounded-xl border border-white/10 bg-surface-900/50 p-6 transition-all duration-200 hover:-translate-y-0.5 hover:border-white/20 hover:shadow-lg hover:shadow-accent/5"
				>
					<div
						class="mb-3 flex h-9 w-9 items-center justify-center rounded-lg bg-surface-800 text-accent"
					>
						{@render featureIcon(feature.icon)}
					</div>
					<h3 class="font-semibold text-surface-50">
						{feature.title}
					</h3>
					<p class="mt-1 text-sm text-surface-400">
						{feature.description}
					</p>
				</div>
			{/each}
		</div>
	</section>

	<!-- How it works -->
	<section
		aria-label="How it works"
		class="mx-auto max-w-6xl px-6 pt-10 pb-16"
	>
		<div class="mb-8">
			<h2 class="text-sm uppercase tracking-widest text-surface-400">
				{$_("hiw.headline")}
			</h2>
			<p class="mt-2 text-sm text-surface-600">
				{$_("hiw.subheadline")}
			</p>
		</div>

		<!-- Steps with connector -->
		<div class="relative">
			<div class="grid gap-6 md:grid-cols-3">
				{#each steps as step (step.number)}
					{@const pill = stepPills[step.number]}
					<div
						class="group relative rounded-xl border border-white/10 bg-surface-900/50 p-6 transition-all duration-200 hover:-translate-y-0.5 hover:border-white/20 hover:shadow-lg hover:shadow-accent/5"
					>
						<!-- Step number badge -->
						<div
							class="mb-4 flex h-9 w-9 items-center justify-center rounded-full border border-accent/20 bg-accent/10 text-sm font-bold text-accent"
						>
							{step.number}
						</div>

						<h3 class="font-semibold text-surface-50">
							{step.title}
						</h3>
						<p class="mt-1 text-sm text-surface-400">
							{step.description}
						</p>

						{#if pill}
							<span
								class="mt-3 inline-flex items-center rounded-full px-2.5 py-0.5 text-[11px] font-medium {pill.color}"
							>
								{stepLabels[step.number]}
							</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>

		<!-- Mini example timeline -->
		<div class="mt-8" aria-hidden="true">
			<div
				class="overflow-hidden rounded-xl border border-white/10 bg-surface-900/50 px-5 py-4"
			>
				<div class="mb-3 flex items-center gap-2">
					<span
						class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>{$_("hiw.example_flow.headline")}</span
					>
					<span
						class="rounded bg-surface-800 px-1.5 py-0.5 text-[10px] text-surface-600"
						>SHP-2617e317...</span
					>
				</div>
				<div
					class="flex flex-col md:flex-row md:items-center md:justify-between md:px-12"
				>
					<!-- Event 1 -->
					<div class="flex items-center gap-2.5 md:flex-1">
						<div
							class="h-2 w-2 shrink-0 rounded-full bg-blue-400"
						></div>
						<div>
							<span class="text-xs text-surface-50"
								>{$_("hiw.example_flow.created")}</span
							>
							<span class="ml-1.5 text-[11px] text-surface-600"
								>{ago(67)}</span
							>
						</div>
					</div>

					<!-- Connector arrow (desktop) -->
					<div
						class="hidden shrink-0 px-2 text-surface-700 md:block"
						aria-hidden="true"
					>
						<svg
							class="h-4 w-4"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<line x1="5" y1="12" x2="19" y2="12" />
							<polyline points="12 5 19 12 12 19" />
						</svg>
					</div>

					<!-- Event 2 -->
					<div class="flex items-center gap-2.5 md:flex-1">
						<div
							class="h-2 w-2 shrink-0 rounded-full bg-amber-400"
						></div>
						<div>
							<span class="text-xs text-surface-50"
								>{$_("hiw.example_flow.in_transit")}</span
							>
							<span class="ml-1.5 text-[11px] text-surface-600"
								>{ago(45)}</span
							>
						</div>
					</div>

					<!-- Connector arrow (desktop) -->
					<div
						class="hidden shrink-0 px-2 text-surface-700 md:block"
						aria-hidden="true"
					>
						<svg
							class="h-4 w-4"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<line x1="5" y1="12" x2="19" y2="12" />
							<polyline points="12 5 19 12 12 19" />
						</svg>
					</div>

					<!-- Event 3 -->
					<div class="flex items-center gap-2.5">
						<div
							class="h-2 w-2 shrink-0 rounded-full bg-accent"
						></div>
						<div>
							<span class="text-xs text-surface-50"
								>{$_("hiw.example_flow.delivered")}</span
							>
							<span class="ml-1.5 text-[11px] text-surface-600"
								>{ago(20)}</span
							>
						</div>
					</div>
				</div>
			</div>
		</div>
	</section>
</main>

<!-- Footer -->
<footer class="border-t border-surface-800 bg-surface-950">
	<div class="mx-auto flex max-w-6xl items-center justify-between px-6 py-6">
		<span class="text-sm text-surface-600">&copy; 2026 LogiPack</span>
		<div class="flex gap-4">
			<a
				href={`/${currentLang}/privacy`}
				class="cursor-pointer text-sm text-surface-600 transition-colors duration-200 hover:text-surface-400 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-950"
			>
				{$_("footer.privacy")}
			</a>
			<a
				href={`/${currentLang}/terms`}
				class="cursor-pointer text-sm text-surface-600 transition-colors duration-200 hover:text-surface-400 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-surface-950"
			>
				{$_("footer.terms")}
			</a>
		</div>
	</div>
</footer>

<style>
	@media (prefers-reduced-motion: reduce) {
		:global(*) {
			transition-duration: 0.01ms !important;
			animation-duration: 0.01ms !important;
		}
	}

	:global(html) {
		scroll-behavior: smooth;
		overscroll-behavior: none;
		overflow-x: hidden;
	}

	main {
		background-image: linear-gradient(
				to right,
				rgba(148, 163, 184, 0.03) 1px,
				transparent 1px
			),
			linear-gradient(
				to bottom,
				rgba(148, 163, 184, 0.03) 1px,
				transparent 1px
			);
		background-size: 40px 40px;
	}
</style>
