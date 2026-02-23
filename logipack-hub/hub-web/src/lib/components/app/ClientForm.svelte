<script lang="ts">
	import { enhance } from "$app/forms";
	import { _ } from "svelte-i18n";

	type FormLike = {
		fieldErrors?: {
			name?: string;
			email?: string;
			phone?: string;
		};
		values?: {
			name?: string;
			email?: string | null;
			phone?: string | null;
		};
		submitError?: string;
	} | null;

	type ClientValues = {
		name: string;
		email?: string | null;
		phone?: string | null;
	};

	let {
		form,
		initialValues,
		cancelHref,
		submitLabelKey = null,
		submitLabel = null,
		loading = false,
	}: {
		form: FormLike;
		initialValues: ClientValues;
		cancelHref: string;
		submitLabelKey?: string | null;
		submitLabel?: string | null;
		loading?: boolean;
	} = $props();

	let submitting = $state(false);

	const enhanceSubmit = () => {
		submitting = true;
		return async ({ update }: { update: () => Promise<void> }) => {
			try {
				await update();
			} finally {
				submitting = false;
			}
		};
	};

	let values = $derived({
		name: form?.values?.name ?? initialValues.name,
		email: form?.values?.email ?? initialValues.email ?? "",
		phone: form?.values?.phone ?? initialValues.phone ?? "",
	});
	let nameError = $derived(form?.fieldErrors?.name ?? null);
	let emailError = $derived(form?.fieldErrors?.email ?? null);
	let phoneError = $derived(form?.fieldErrors?.phone ?? null);
	let submitError = $derived(form?.submitError ?? null);
	let resolvedSubmitLabel = $derived(
		submitLabelKey
			? $_(submitLabelKey)
			: submitLabel ?? $_("admin.clients.form.submit"),
	);
	let isBusy = $derived(loading || submitting);
</script>

<section
	class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5 sm:p-6"
>
	<form method="POST" class="space-y-5" use:enhance={enhanceSubmit}>
		{#if submitError}
			<div
				class="rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-300"
				aria-live="polite"
			>
				{$_(submitError)}
			</div>
		{/if}

		<fieldset class="m-0 min-w-0 space-y-4 border-0 p-0">
			<div class="grid gap-4 md:grid-cols-2">
				<div class="space-y-1.5">
					<label for="name" class="text-sm font-medium text-surface-200">
						{$_("admin.clients.form.name")}
						<span class="text-red-400">*</span>
					</label>
					<input
						id="name"
						name="name"
						type="text"
						value={values.name}
						placeholder={$_("admin.clients.form.name_placeholder")}
						required
						autocomplete="organization"
						aria-invalid={nameError ? "true" : undefined}
						aria-describedby={nameError
							? "client_name_hint client_name_error"
							: "client_name_hint"}
						disabled={isBusy}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							nameError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					/>
					<p id="client_name_hint" class="text-xs text-surface-400">
						{$_("admin.clients.form.name_hint")}
					</p>
					<p
						id="client_name_error"
						class="min-h-4 text-xs text-red-400"
						aria-live="polite"
					>
						{nameError ? $_(nameError) : ""}
					</p>
				</div>

				<div class="space-y-1.5">
					<label for="email" class="text-sm font-medium text-surface-200">
						{$_("admin.clients.form.email")}
					</label>
					<input
						id="email"
						name="email"
						type="email"
						value={values.email}
						placeholder={$_("admin.clients.form.email_placeholder")}
						autocomplete="email"
						aria-invalid={emailError ? "true" : undefined}
						aria-describedby={emailError
							? "client_email_hint client_email_error"
							: "client_email_hint"}
						disabled={isBusy}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							emailError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					/>
					<p id="client_email_hint" class="text-xs text-surface-400">
						{$_("admin.clients.form.email_hint")}
					</p>
					<p
						id="client_email_error"
						class="min-h-4 text-xs text-red-400"
						aria-live="polite"
					>
						{emailError ? $_(emailError) : ""}
					</p>
				</div>

				<div class="space-y-1.5 md:col-span-2">
					<label for="phone" class="text-sm font-medium text-surface-200">
						{$_("admin.clients.form.phone")}
					</label>
					<input
						id="phone"
						name="phone"
						type="tel"
						value={values.phone}
						placeholder={$_("admin.clients.form.phone_placeholder")}
						autocomplete="tel"
						aria-invalid={phoneError ? "true" : undefined}
						aria-describedby={phoneError
							? "client_phone_hint client_phone_error"
							: "client_phone_hint"}
						disabled={isBusy}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							phoneError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					/>
					<p id="client_phone_hint" class="text-xs text-surface-400">
						{$_("admin.clients.form.phone_hint")}
					</p>
					<p
						id="client_phone_error"
						class="min-h-4 text-xs text-red-400"
						aria-live="polite"
					>
						{phoneError ? $_(phoneError) : ""}
					</p>
				</div>
			</div>
		</fieldset>

		<div
			class="mt-1 flex flex-col-reverse gap-2 pt-4 sm:flex-row sm:justify-end"
		>
			<a
				href={cancelHref}
				class="inline-flex cursor-pointer items-center justify-center rounded-lg border border-surface-700 bg-surface-800 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors hover:bg-surface-700 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				{$_("admin.clients.form.cancel")}
			</a>
			<button
				type="submit"
				disabled={isBusy}
				class="inline-flex cursor-pointer items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:bg-surface-700 disabled:text-surface-400"
			>
				{isBusy ? $_("admin.clients.form.submitting") : resolvedSubmitLabel}
			</button>
		</div>
	</form>
</section>
