<script lang="ts">
	import { _ } from "svelte-i18n";
	import NavItemIcon from "$lib/components/app/NavItemIcon.svelte";

	let { pathname, lang }: { pathname: string; lang: string } = $props();
	type IconName =
		| "dashboard"
		| "shipments"
		| "reports"
		| "clients"
		| "offices"
		| "employees"
		| "audit";
	type MatchMode = "exact" | "prefix";

	interface NavItem {
		labelKey: string;
		href: string;
		icon: IconName;
		match: MatchMode;
	}

	interface NavSection {
		labelKey: string;
		items: NavItem[];
	}

	function trimTrailingSlash(path: string): string {
		if (path.length <= 1) return path;
		return path.replace(/\/+$/, "");
	}

	let normalizedPathname = $derived(trimTrailingSlash(pathname));
	let adminBase = $derived(`/${lang}/app/admin`);
	let employeeBase = $derived(`/${lang}/app/employee`);
	let adminProfileHref = $derived(`${adminBase}/profile`);
	let employeeProfileHref = $derived(`${employeeBase}/profile`);
	let isAdminConsole = $derived(
		normalizedPathname === adminBase ||
			normalizedPathname.startsWith(`${adminBase}/`),
	);
	let isAdminProfileActive = $derived(
		normalizedPathname === adminProfileHref ||
			normalizedPathname.startsWith(`${adminProfileHref}/`),
	);
	let isEmployeeProfileActive = $derived(
		normalizedPathname === employeeProfileHref ||
			normalizedPathname.startsWith(`${employeeProfileHref}/`),
	);
	let profileHref = $derived(
		isAdminConsole ? adminProfileHref : employeeProfileHref,
	);
	let isProfileActive = $derived(
		isAdminConsole ? isAdminProfileActive : isEmployeeProfileActive,
	);

	const navSections: NavSection[] = $derived.by(() => {
		if (isAdminConsole) {
			return [
				{
					labelKey: "navbar.section.operations",
					items: [
						{
							labelKey: "navbar.item.dashboard",
							href: adminBase,
							icon: "dashboard",
							match: "exact",
						},
						{
							labelKey: "navbar.item.shipments",
							href: `${adminBase}/shipments`,
							icon: "shipments",
							match: "prefix",
						},
						{
							labelKey: "navbar.item.reports",
							href: `${adminBase}/reports`,
							icon: "reports",
							match: "prefix",
						},
					],
				},
				{
					labelKey: "navbar.section.administration",
					items: [
						{
							labelKey: "navbar.item.clients",
							href: `${adminBase}/clients`,
							icon: "clients",
							match: "prefix",
						},
						{
							labelKey: "navbar.item.offices",
							href: `${adminBase}/offices`,
							icon: "offices",
							match: "prefix",
						},
						{
							labelKey: "navbar.item.employees",
							href: `${adminBase}/employees`,
							icon: "employees",
							match: "prefix",
						},
						{
							labelKey: "navbar.item.audit_log",
							href: `${adminBase}/audit`,
							icon: "audit",
							match: "prefix",
						},
					],
				},
			];
		}

		return [
			{
				labelKey: "navbar.section.operations",
				items: [
					{
						labelKey: "navbar.item.dashboard",
						href: employeeBase,
						icon: "dashboard",
						match: "exact",
					},
					{
						labelKey: "navbar.item.shipments",
						href: `${employeeBase}/shipments`,
						icon: "shipments",
						match: "prefix",
					},
				],
			},
		];
	});

	function isActive(item: NavItem): boolean {
		const targetPath = trimTrailingSlash(item.href);
		if (item.match === "exact") {
			return normalizedPathname === targetPath;
		}
		return (
			normalizedPathname === targetPath ||
			normalizedPathname.startsWith(`${targetPath}/`)
		);
	}
</script>

<aside
	class="flex h-full w-60 flex-col border-r border-surface-700/50 bg-surface-900"
>
	<!-- Brand block -->
	<div class="px-5 pt-4.25 pb-3.5">
		<div class="flex items-center gap-2.5">
			<img
				src="https://raw.githubusercontent.com/Emagjby/logipack-assets/refs/heads/main/logipack-crate-green.png"
				alt="LogiPack"
				class="h-6 w-6 rounded-sm object-cover"
			/>
			<span class="text-sm font-semibold tracking-tight text-surface-50"
				>LogiPack</span
			>
		</div>
	</div>

	<!-- Navigation section -->
	<nav class="flex-1 overflow-y-auto px-3 py-4">
		{#each navSections as section, sectionIndex (section.labelKey)}
			<div
				class={sectionIndex === navSections.length - 1
					? "mb-0"
					: "mb-5"}
			>
				<span
					class="mb-2 block px-3 text-[10px] font-medium uppercase tracking-widest text-surface-600"
				>
					{$_(section.labelKey)}
				</span>

				<div class="space-y-1">
					{#each section.items as item (item.href)}
						{@const active = isActive(item)}
						<a
							href={item.href}
							class={[
								"group relative flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-[13px] font-medium transition-all duration-150",
								"focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 focus-visible:ring-offset-1 focus-visible:ring-offset-surface-900",
								active
									? "bg-surface-800/80 text-surface-50"
									: "text-surface-400 hover:bg-surface-800/40 hover:text-surface-200",
							]}
						>
							{#if active}
								<div
									class="absolute left-0 top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-full bg-accent"
								></div>
							{/if}

							<NavItemIcon name={item.icon} {active} />

							<span>{$_(item.labelKey)}</span>
						</a>
					{/each}
				</div>
			</div>
		{/each}
	</nav>

	<!-- Bottom section -->
	<div class="mt-auto px-3 py-3">
		<span
			class="mb-2 block px-3 text-[10px] font-medium uppercase tracking-widest text-surface-600"
		>
			{$_("navbar.section.account")}
		</span>
		<a
			href={profileHref}
			class={[
				"group relative flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-[13px] font-medium transition-all duration-150",
				"focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 focus-visible:ring-offset-1 focus-visible:ring-offset-surface-900",
				isProfileActive
					? "bg-surface-800/80 text-surface-50"
					: "text-surface-400 hover:bg-surface-800/40 hover:text-surface-200",
			]}
		>
			{#if isProfileActive}
				<div
					class="absolute left-0 top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-full bg-accent"
				></div>
			{/if}
			<NavItemIcon name="profile" active={isProfileActive} />
			<span>{$_("navbar.item.profile")}</span>
		</a>
		<span class="mt-2 block px-2 text-[10px] text-surface-700">v0.1.0</span>
	</div>
</aside>
