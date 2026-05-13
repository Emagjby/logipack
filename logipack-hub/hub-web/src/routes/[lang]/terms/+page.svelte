<script lang="ts">
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";

	const lang = $derived(page.params.lang ?? "en");
	const backHref = $derived(`/${lang}`);
</script>

<div class="min-h-screen bg-surface-950 text-surface-50">
	<main class="mx-auto flex min-h-[70vh] w-full max-w-4xl flex-col gap-8 px-6 py-16">
		<a
			href={backHref}
			class="inline-flex w-fit items-center gap-2 rounded-lg border border-surface-800 bg-surface-950/80 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors duration-150 hover:border-surface-600 hover:bg-surface-900/80 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
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
				<polyline points="15 18 9 12 15 6" />
			</svg>
			{$_("legal.back")}
		</a>

		<header class="space-y-3 rounded-2xl border border-surface-800 bg-surface-950/80 p-6 shadow-lg shadow-black/25">
			<p class="text-xs uppercase tracking-[0.2em] text-surface-500">
				{$_("legal.kicker")}
			</p>
			<h1 class="text-3xl font-semibold text-surface-50">
				{$_("terms.title")}
			</h1>
			<p class="text-sm text-surface-400">
				{$_("terms.subtitle")}
			</p>
		</header>

		<div class="grid gap-4">
			<section class="space-y-3 rounded-xl border border-surface-800 bg-surface-950/70 p-6">
				<h2 class="text-lg font-semibold text-surface-50">
					{$_("terms.section.demo.title")}
				</h2>
				<p class="text-sm leading-relaxed text-surface-400">
					{$_("terms.section.demo.body")}
				</p>
			</section>

			<section class="space-y-3 rounded-xl border border-surface-800 bg-surface-950/70 p-6">
				<h2 class="text-lg font-semibold text-surface-50">
					{$_("terms.section.usage.title")}
				</h2>
				<p class="text-sm leading-relaxed text-surface-400">
					{$_("terms.section.usage.body")}
				</p>
			</section>

			<section class="space-y-3 rounded-xl border border-surface-800 bg-surface-950/70 p-6">
				<h2 class="text-lg font-semibold text-surface-50">
					{$_("terms.section.license.title")}
				</h2>
				<p class="text-sm leading-relaxed text-surface-400">
					{$_("terms.section.license.body")}
				</p>
			</section>
		</div>
	</main>
</div>
