<script lang="ts">
	import { onDestroy } from "svelte";
	import { _ } from "svelte-i18n";
	import {
		formatDateTime,
		shortenHash,
		type StrataPackage,
	} from "$lib/domain/strataPackage";

	let {
		pkg,
		lang,
		onclose,
	}: {
		pkg: StrataPackage | null;
		lang: string;
		onclose: () => void;
	} = $props();
	let copiedStrata = $state(false);
	let copiedTimer = $state<ReturnType<typeof setTimeout> | null>(null);

	function handleKeydown(e: KeyboardEvent) {
		if (!pkg) return;
		if (e.key === "Escape") {
			onclose();
		}
	}

	function clearCopiedTimer(): void {
		if (!copiedTimer) return;
		clearTimeout(copiedTimer);
		copiedTimer = null;
	}

	onDestroy(() => {
		clearCopiedTimer();
	});

	async function handleCopyStrata(): Promise<void> {
		if (!pkg) return;

		try {
			await navigator.clipboard.writeText(
				JSON.stringify(pkg.payload_json, null, 2),
			);
			copiedStrata = true;
			clearCopiedTimer();
			copiedTimer = setTimeout(() => {
				copiedStrata = false;
				copiedTimer = null;
			}, 1200);
		} catch {
			// Ignore clipboard errors.
		}
	}
</script>

<svelte:document onkeydown={handleKeydown} />

{#if pkg}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- Backdrop -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
		onclick={onclose}
	>
		<!-- Modal panel -->
		<div
			class="relative mx-4 flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-label={$_("shipment.viewer.title")}
			tabindex="-1"
		>
			<!-- Header -->
			<div
				class="flex items-center justify-between border-b border-surface-700/50 px-6 py-4"
			>
				<h2 class="text-base font-semibold text-surface-200">
					{$_("shipment.viewer.title")}
				</h2>
				<button
					onclick={onclose}
					class="rounded-lg p-1.5 text-surface-400 cursor-pointer transition-colors hover:bg-surface-800 hover:text-surface-200"
					aria-label={$_("shipment.viewer.close")}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="h-4 w-4"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<!-- Scrollable body -->
			<div class="flex-1 overflow-y-auto px-6 py-5">
				<!-- Metadata grid -->
				<div class="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3">
					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.hash")}
						</p>
						<p
							class="mt-1 font-mono text-xs text-accent"
							title={pkg.hash}
						>
							{shortenHash(pkg.hash)}
						</p>
					</div>

					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.prev_hash")}
						</p>
						<p
							class={[
								"mt-1 font-mono text-xs",
								pkg.prev_hash
									? "text-accent"
									: "text-surface-500 italic",
							]}
							title={pkg.prev_hash ?? undefined}
						>
							{pkg.prev_hash
								? shortenHash(pkg.prev_hash)
								: $_("common.none")}
						</p>
					</div>

					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.stream_id")}
						</p>
						<p
							class="mt-1 font-mono text-xs text-accent"
							title={pkg.stream_id}
						>
							{shortenHash(pkg.stream_id)}
						</p>
					</div>

					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.seq")}
						</p>
						<p class="mt-1 font-mono text-xs text-surface-200">
							{pkg.seq}
						</p>
					</div>

					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.event_type")}
						</p>
						<p class="mt-1 font-mono text-xs text-surface-200">
							{pkg.event_type}
						</p>
					</div>

					<div>
						<p
							class="text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("shipment.strata.created_at")}
						</p>
						<p class="mt-1 text-xs text-surface-200">
							{formatDateTime(pkg.created_at, lang)}
						</p>
					</div>
				</div>

				<!-- Payload section -->
				<div class="mt-6">
					<h3 class="mb-3 text-sm font-medium text-surface-200">
						{$_("shipment.viewer.payload")}
					</h3>
					<pre
						class="overflow-x-auto rounded-lg bg-surface-800 p-4 font-mono text-xs text-surface-200">{JSON.stringify(
							pkg.payload_json,
							null,
							2,
						)}</pre>
				</div>
			</div>

			<!-- Footer -->
			<div
				class="flex items-center justify-end gap-3 border-t border-surface-700/50 px-6 py-4"
			>
				<button
					type="button"
					onclick={handleCopyStrata}
					class="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-surface-950 transition-colors hover:bg-accent-hover"
					aria-live="polite"
				>
					<span>{$_("shipment.viewer.copy_strata")}</span>
					<span
						class="inline-flex h-3.5 w-3.5 items-center justify-center"
						aria-hidden="true"
					>
						{#if copiedStrata}
							<svg
								class="h-3.5 w-3.5"
								viewBox="0 0 20 20"
								fill="currentColor"
							>
								<path
									fill-rule="evenodd"
									d="M16.704 5.29a1 1 0 010 1.42l-8 8a1 1 0 01-1.42 0l-4-4a1 1 0 111.42-1.42L8 12.59l7.29-7.3a1 1 0 011.414 0z"
									clip-rule="evenodd"
								/>
							</svg>
						{/if}
					</span>
				</button>
				<button
					onclick={onclose}
					class="rounded-lg bg-surface-800 px-4 py-2 cursor-pointer text-sm font-medium text-surface-400 transition-colors hover:bg-surface-700"
				>
					{$_("shipment.viewer.close")}
				</button>
			</div>
		</div>
	</div>
{/if}
