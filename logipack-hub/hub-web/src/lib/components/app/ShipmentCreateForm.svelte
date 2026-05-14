<script lang="ts">
	import { enhance } from "$app/forms";
	import { _ } from "svelte-i18n";

	type ClientOption = {
		id: string;
		name: string;
		email?: string | null;
	};

	type OfficeOption = {
		id: string;
		name: string;
		city: string;
	};

	type FormLike = {
		fieldErrors?: {
			client_id?: string;
		};
		values?: {
			client_id?: string;
			current_office_id?: string;
			notes?: string;
		};
		submitError?: string;
	} | null;

	let {
		form,
		cancelHref,
		showOfficeInput = true,
		lockedOfficeId = null,
		officeBadgeText = null,
		clients = [],
		offices = [],
		loading = false,
	}: {
		form: FormLike;
		cancelHref: string;
		showOfficeInput?: boolean;
		lockedOfficeId?: string | null;
		officeBadgeText?: string | null;
		clients?: ClientOption[];
		offices?: OfficeOption[];
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
		client_id: form?.values?.client_id ?? "",
		current_office_id:
			form?.values?.current_office_id ?? lockedOfficeId ?? "",
		notes: form?.values?.notes ?? "",
	});
	let clientIdError = $derived(form?.fieldErrors?.client_id ?? null);
	let submitError = $derived(form?.submitError ?? null);
	let isBusy = $derived(loading || submitting);

	function clientLabel(client: ClientOption): string {
		return client.email ? `${client.name} (${client.email})` : client.name;
	}

	function officeLabel(office: OfficeOption): string {
		return `${office.name} (${office.city})`;
	}
</script>

<section class="stagger stagger-1">
	<h1 class="text-2xl font-bold text-surface-50">
		{$_("admin.shipments.new.headline")}
	</h1>
	<p class="mt-1 max-w-2xl text-sm text-surface-400">
		{$_("admin.shipments.new.subtitle")}
	</p>
</section>

<section
	class="stagger stagger-2 mt-6 rounded-xl border border-surface-700/50 bg-surface-900 p-5 sm:p-6"
>
	<form method="POST" class="space-y-5" use:enhance={enhanceSubmit}>
		<div class="flex flex-wrap items-center gap-2">
			<span
				class="rounded-full bg-surface-800 px-2.5 py-1 text-xs font-medium text-surface-400"
			>
				{$_("admin.shipments.new.context.status_auto")}
			</span>
			{#if officeBadgeText}
				<span
					class="rounded-full bg-surface-800 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{officeBadgeText}
				</span>
			{/if}
			{#if loading}
				<span
					class="rounded-full bg-surface-800 px-2.5 py-1 text-xs font-medium text-surface-400"
				>
					{$_("employee.shipments.new.loading_office")}
				</span>
			{/if}
		</div>

		{#if submitError}
			<div
				class="rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-300"
				aria-live="polite"
			>
				{$_(submitError)}
			</div>
		{/if}

		<div class={showOfficeInput ? "grid gap-4 md:grid-cols-2" : "grid gap-4"}>
			<div class="space-y-1.5">
				<label
					for="client_id"
					class="text-sm font-medium text-surface-200"
				>
					{$_("admin.shipments.new.client_id")}
					<span class="text-red-400">*</span>
				</label>
				{#if clients.length > 0}
					<select
						id="client_id"
						name="client_id"
						value={values.client_id}
						required
						aria-invalid={clientIdError ? "true" : undefined}
						aria-describedby={clientIdError ? "client_id_error" : undefined}
						disabled={isBusy}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							clientIdError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					>
						<option value="" disabled>{$_("admin.shipments.new.select_client")}</option>
						{#each clients as client (client.id)}
							<option value={client.id}>{clientLabel(client)}</option>
						{/each}
					</select>
				{:else}
					<input
						id="client_id"
						name="client_id"
						type="text"
						value={values.client_id}
						aria-invalid={clientIdError ? "true" : undefined}
						aria-describedby={clientIdError ? "client_id_error" : undefined}
						disabled={isBusy}
						class={[
							"w-full rounded-lg border bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 disabled:cursor-not-allowed disabled:opacity-70",
							clientIdError
								? "border-red-500/70 focus-visible:ring-red-400/60"
								: "border-surface-700 focus-visible:ring-accent/50",
						]}
					/>
				{/if}
				{#if clientIdError}
					<p
						id="client_id_error"
						class="text-xs text-red-400"
						aria-live="polite"
					>
						{$_(clientIdError)}
					</p>
				{/if}
			</div>

			{#if showOfficeInput}
				<div class="space-y-1.5">
					<label
						for="current_office_id"
						class="text-sm font-medium text-surface-200"
					>
						{$_("admin.shipments.new.current_office_id")}
						<span class="text-xs font-normal text-surface-400">
							({$_("admin.shipments.new.optional")})
						</span>
					</label>
					{#if offices.length > 0}
						<select
							id="current_office_id"
							name="current_office_id"
							value={values.current_office_id}
							disabled={isBusy}
							class="w-full rounded-lg border border-surface-700 bg-surface-800 px-3 py-2 text-sm text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:opacity-70"
						>
							<option value="">{$_("admin.shipments.new.no_office")}</option>
							{#each offices as office (office.id)}
								<option value={office.id}>{officeLabel(office)}</option>
							{/each}
						</select>
					{:else}
						<input
							id="current_office_id"
							name="current_office_id"
							type="text"
							value={values.current_office_id}
							disabled={isBusy}
							class="w-full rounded-lg border border-surface-700 bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:opacity-70"
						/>
					{/if}
					<p class="text-xs text-surface-400">
						{$_("admin.shipments.new.current_office_hint")}
					</p>
				</div>
			{:else}
				<input
					type="hidden"
					name="current_office_id"
					value={lockedOfficeId ?? values.current_office_id}
				/>
			{/if}
		</div>

		<div class="space-y-1.5">
			<label for="notes" class="text-sm font-medium text-surface-200">
				{$_("admin.shipments.new.notes")}
				<span class="text-xs font-normal text-surface-400">
					({$_("admin.shipments.new.optional")})
				</span>
			</label>
			<textarea
				id="notes"
				name="notes"
				rows="5"
				placeholder={$_("admin.shipments.new.notes_placeholder")}
				disabled={isBusy}
				class="min-h-[120px] w-full rounded-lg border border-surface-700 bg-surface-800 px-3 py-2 text-sm text-surface-200 placeholder:text-surface-400 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:opacity-70"
				>{values.notes}</textarea
			>
		</div>

		<div
			class="mt-1 flex flex-col-reverse gap-2 pt-4 sm:flex-row sm:justify-end"
		>
			<a
				href={cancelHref}
				class="inline-flex cursor-pointer items-center justify-center rounded-lg border border-surface-700 bg-surface-800 px-4 py-2 text-sm font-semibold text-surface-200 transition-colors hover:bg-surface-700 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				{$_("admin.shipments.new.cancel")}
			</a>
			<button
				type="submit"
				disabled={isBusy}
				class="inline-flex cursor-pointer items-center justify-center rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 disabled:cursor-not-allowed disabled:bg-surface-700 disabled:text-surface-400"
			>
				{$_("admin.shipments.new.create")}
			</button>
		</div>
	</form>
</section>
