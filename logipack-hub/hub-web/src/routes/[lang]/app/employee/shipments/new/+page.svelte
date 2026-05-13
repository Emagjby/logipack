<script lang="ts">
	import { page } from "$app/state";
	import ShipmentCreateForm from "$lib/components/app/ShipmentCreateForm.svelte";
	import { _ } from "svelte-i18n";
	import type { ActionData, PageData } from "./$types";

	let {
		data,
		form,
	}: {
		data: PageData;
		form: ActionData | null;
	} = $props();

	let lang = $derived(page.params.lang || "en");

	let officePillValue = $derived.by(() =>
		data.office.isAvailable
			? (data.office.label ?? data.office.assignedId)
			: $_("employee.shipments.new.office_not_available"),
	);

	let officePillText = $derived(
		`${$_("shipment.form.office")}: ${officePillValue}`,
	);
</script>

<ShipmentCreateForm
	{form}
	clients={data.clients}
	cancelHref={`/${lang}/app/employee/shipments`}
	showOfficeInput={false}
	lockedOfficeId={data.office.assignedId}
	officeBadgeText={officePillText}
	loading={data.office.isLoading}
/>
