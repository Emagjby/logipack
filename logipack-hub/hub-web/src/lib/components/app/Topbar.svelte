<script lang="ts">
	import { onDestroy } from "svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { _ } from "svelte-i18n";

	let {
		pathname,
		session,
		lang,
	}: {
		pathname: string;
		session: { name: string; email: string; role?: string } | null;
		lang: string;
	} = $props();

	function formatSegment(seg: string): string {
		if (/^[0-9a-f]{8,}$/i.test(seg) || /^\d+$/.test(seg)) {
			return seg.length > 8 ? seg.slice(0, 8) + "..." : seg;
		}
		return seg.charAt(0).toUpperCase() + seg.slice(1);
	}

	const segmentLabelKeys: Record<string, string> = {
		admin: "navbar.page.dashboard",
		employee: "navbar.page.dashboard",
		shipments: "navbar.page.shipments",
		profile: "navbar.page.profile",
		clients: "navbar.page.clients",
		offices: "navbar.page.offices",
		employees: "navbar.page.employees",
		audit: "navbar.page.audit_log",
		reports: "navbar.page.reports",
	};

	const roleLabelKeys: Record<string, string> = {
		employee: "navbar.role.employee",
		admin: "navbar.role.admin",
	};

	function getSegmentLabel(seg: string): string {
		const key = segmentLabelKeys[seg.toLowerCase()];
		return key ? $_(key) : formatSegment(seg);
	}

	function getRoleLabel(role: string): string {
		const key = roleLabelKeys[role.toLowerCase()];
		return key ? $_(key) : role;
	}

	let segments = $derived(
		pathname
			.replace(new RegExp(`^/${lang}/app/?`), "")
			.split("/")
			.filter(Boolean),
	);

	let pageTitleSegment = $derived(
		segments.length === 0 ? null : segments[segments.length - 1],
	);
	let normalizedPageTitleSegment = $derived.by(() => {
		if (
			segments.length >= 2 &&
			segments[segments.length - 1]?.toLowerCase() === "new"
		) {
			return (
				segments[segments.length - 2]?.toLowerCase() ?? pageTitleSegment
			);
		}
		return pageTitleSegment;
	});

	let userEmail = $derived(session?.email ?? "user@unknown");
	let userName = $derived(session?.name ?? "User");
	let userInitial = $derived(
		(session?.name ?? session?.email ?? "U").charAt(0).toUpperCase(),
	);
	let userRole = $derived(session?.role ?? "employee");

	let dropdownOpen = $state(false);
	let searchValue = $state("");
	let searchTimer: ReturnType<typeof setTimeout> | null = null;
	let isSearchFocused = $state(false);

	let isShipmentsListPage = $derived(
		/^\/[^/]+\/app\/employee\/shipments\/?$/.test(pathname),
	);
	let isAdminOfficesListPage = $derived(
		/^\/[^/]+\/app\/admin\/offices\/?$/.test(pathname),
	);
	let isSearchEnabled = $derived(
		isShipmentsListPage || isAdminOfficesListPage,
	);
	let querySearchValue = $derived(page.url.searchParams.get("q") ?? "");

	$effect(() => {
		if (!isSearchEnabled) {
			searchValue = "";
			return;
		}
		if (isSearchFocused) return;
		searchValue = querySearchValue;
	});

	onDestroy(() => {
		if (searchTimer) {
			clearTimeout(searchTimer);
			searchTimer = null;
		}
	});

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest("[data-user-menu]")) {
			dropdownOpen = false;
		}
	}

	function logout() {
		window.location.href = `/logout`;
	}

	function updateTopbarSearch(next: string) {
		searchValue = next;
		if (!isSearchEnabled) return;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(async () => {
			const url = new URL(page.url);
			const trimmed = next.trim();
			if (trimmed) {
				url.searchParams.set("q", trimmed);
			} else {
				url.searchParams.delete("q");
			}
			const nextHref = `${url.pathname}${url.search}`;
			const currentHref = `${page.url.pathname}${page.url.search}`;
			if (nextHref === currentHref) {
				searchTimer = null;
				return;
			}
			await goto(nextHref, {
				replaceState: true,
				keepFocus: true,
				noScroll: true,
			});
			searchTimer = null;
		}, 180);
	}
</script>

<svelte:document onclick={handleClickOutside} />

<header
	class="sticky top-0 z-50 flex h-14 shrink-0 items-center justify-between border-b border-surface-800 bg-surface-900/95 backdrop-blur-sm px-6"
>
	<!-- Left side: Page title -->
	<div class="flex flex-col justify-center min-w-0">
		<h1 class="text-sm font-semibold text-surface-50 truncate">
			{#if normalizedPageTitleSegment === null}
				{$_("navbar.page.overview")}
			{:else}
				{getSegmentLabel(normalizedPageTitleSegment)}
			{/if}
		</h1>
	</div>

	<!-- Right side: Search + User chip + Logout -->
	<div class="flex items-center gap-3">
		<!-- Search input -->
		<div class="relative hidden sm:block">
			<svg
				class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-surface-600 pointer-events-none"
				viewBox="0 0 20 20"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="8.5" cy="8.5" r="5.5" />
				<path d="M13 13l4 4" />
			</svg>
			<input
				type="text"
				value={searchValue}
				onfocus={() => {
					isSearchFocused = true;
				}}
				onblur={() => {
					isSearchFocused = false;
				}}
				oninput={(e) =>
					updateTopbarSearch(
						(e.currentTarget as HTMLInputElement).value,
					)}
				placeholder={isAdminOfficesListPage
					? $_("admin.offices.search_placeholder")
					: isShipmentsListPage
						? $_("shipments.search_placeholder")
						: $_("navbar.search_placeholder")}
				disabled={!isSearchEnabled}
				class="h-8 w-44 rounded-md border border-surface-700/50 bg-surface-800/50 pl-8 pr-3 text-xs text-surface-400 placeholder:text-surface-600 focus:outline-none disabled:cursor-not-allowed"
			/>
		</div>

		<!-- Separator -->
		<div class="hidden sm:block h-5 w-px bg-surface-800"></div>

		<!-- User menu -->
		<div class="relative z-50" data-user-menu>
			<button
				onclick={() => (dropdownOpen = !dropdownOpen)}
				class="flex cursor-pointer items-center gap-2 rounded-lg bg-surface-800/60 py-1.5 pl-1.5 pr-3 transition-colors duration-150 hover:bg-surface-800 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
			>
				<!-- Avatar circle -->
				<div
					class="flex h-6 w-6 items-center justify-center rounded-md bg-accent/15 text-[11px] font-semibold text-accent"
				>
					{userInitial}
				</div>
				<!-- Email -->
				<span
					class="hidden text-xs text-surface-400 pb-0.5 sm:block max-w-40 truncate"
					>{userEmail}</span
				>
				<!-- Chevron -->
				<svg
					class={[
						"h-3.5 w-3.5 text-surface-600 hidden sm:block transition-transform duration-150",
						dropdownOpen && "rotate-180",
					]}
					viewBox="0 0 20 20"
					fill="currentColor"
				>
					<path
						fill-rule="evenodd"
						d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
						clip-rule="evenodd"
					/>
				</svg>
			</button>

			<!-- Dropdown -->
			{#if dropdownOpen}
				<div
					class="absolute right-0 top-full z-50 mt-1.5 w-56 rounded-lg border border-surface-700/50 bg-surface-900 shadow-lg shadow-black/30"
				>
					<!-- User info -->
					<div class="border-b border-surface-800 px-3 py-2.5">
						<p
							class="text-xs font-medium text-surface-200 truncate"
						>
							{userName}
						</p>
						<p class="text-[11px] text-surface-600 truncate">
							{userEmail}
						</p>
						<span
							class="mt-1 inline-block rounded-full bg-surface-800 px-2 py-0.5 text-[10px] font-medium capitalize text-surface-400"
							>{getRoleLabel(userRole)}</span
						>
					</div>

					<!-- Actions -->
					<div class="p-1">
						<button
							onclick={logout}
							class="flex w-full cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-2 text-xs text-surface-400 transition-colors duration-150 hover:bg-surface-800 hover:text-surface-200 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
						>
							<svg
								class="h-4 w-4 shrink-0 text-surface-400"
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 640 640"
								><!--!Font Awesome Free v7.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2026 Fonticons, Inc.--><path
									d="M224 160C241.7 160 256 145.7 256 128C256 110.3 241.7 96 224 96L160 96C107 96 64 139 64 192L64 448C64 501 107 544 160 544L224 544C241.7 544 256 529.7 256 512C256 494.3 241.7 480 224 480L160 480C142.3 480 128 465.7 128 448L128 192C128 174.3 142.3 160 160 160L224 160zM566.6 342.6C579.1 330.1 579.1 309.8 566.6 297.3L438.6 169.3C426.1 156.8 405.8 156.8 393.3 169.3C380.8 181.8 380.8 202.1 393.3 214.6L466.7 288L256 288C238.3 288 224 302.3 224 320C224 337.7 238.3 352 256 352L466.7 352L393.3 425.4C380.8 437.9 380.8 458.2 393.3 470.7C405.8 483.2 426.1 483.2 438.6 470.7L566.6 342.7z"
									fill="currentColor"
								/></svg
							>
							{$_("navbar.sign_out")}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
</header>
