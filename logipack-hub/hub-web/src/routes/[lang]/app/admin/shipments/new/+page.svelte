<script lang="ts">
	import { page } from "$app/state";
	import ShipmentCreateForm from "$lib/components/app/ShipmentCreateForm.svelte";
	import type { ActionData, PageData } from "./$types";

	let { data, form }: { data: PageData; form: ActionData | null } = $props();
	let lang = $derived(page.params.lang || "en");
</script>

<ShipmentCreateForm
	{form}
	clients={data.clients}
	offices={data.offices}
	cancelHref={`/${lang}/app/admin/shipments`}
	showOfficeInput
/>
