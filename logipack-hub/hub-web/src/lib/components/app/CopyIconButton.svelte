<script lang="ts">
	import { onDestroy } from "svelte";

	type Props = {
		value: string;
		title?: string;
		ariaLabel?: string;
		class?: string;
		stopPropagation?: boolean;
		disabled?: boolean;
		onCopied?: () => void;
	};

	let {
		value,
		title = "Copy",
		ariaLabel = title,
		class: className = "",
		stopPropagation = false,
		disabled = false,
		onCopied,
	}: Props = $props();

	let copied = $state(false);
	let copyTimer = $state<ReturnType<typeof setTimeout> | null>(null);

	function clearCopyTimer(): void {
		if (copyTimer) {
			clearTimeout(copyTimer);
			copyTimer = null;
		}
	}

	onDestroy(() => {
		clearCopyTimer();
	});

	async function handleCopy(event: MouseEvent): Promise<void> {
		if (stopPropagation) {
			event.preventDefault();
			event.stopPropagation();
		}

		if (disabled) return;

		try {
			await navigator.clipboard.writeText(value);
			copied = true;
			onCopied?.();
			clearCopyTimer();
			copyTimer = setTimeout(() => {
				copied = false;
				copyTimer = null;
			}, 1200);
		} catch {
			// Ignore clipboard errors.
		}
	}
</script>

<button
	type="button"
	onclick={handleCopy}
	onkeydown={(event) => {
		if (stopPropagation) event.stopPropagation();
	}}
	{title}
	aria-label={ariaLabel}
	{disabled}
	class={[
		"inline-flex h-6 w-6 items-center justify-center rounded-md bg-surface-800 text-surface-400 transition-colors hover:bg-surface-700 hover:text-accent disabled:cursor-not-allowed disabled:opacity-60",
		className,
	]}
>
	{#if copied}
		<svg
			class="h-3.5 w-3.5 text-emerald-400"
			viewBox="0 0 20 20"
			fill="currentColor"
		>
			<path
				fill-rule="evenodd"
				d="M16.704 5.29a1 1 0 010 1.42l-8 8a1 1 0 01-1.42 0l-4-4a1 1 0 111.42-1.42L8 12.59l7.29-7.3a1 1 0 011.414 0z"
				clip-rule="evenodd"
			/>
		</svg>
	{:else}
		<svg
			class="h-3.5 w-3.5"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<rect x="9" y="9" width="11" height="11" rx="2" />
			<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
		</svg>
	{/if}
</button>
