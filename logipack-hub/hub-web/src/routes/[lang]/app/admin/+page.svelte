<script lang="ts">
	import type { PageData } from "./$types";
	import { goto, invalidateAll } from "$app/navigation";
	import { _ } from "svelte-i18n";
	import { onDestroy } from "svelte";

	let { data }: { data: PageData } = $props();

	let lang = $derived(data.pathname.split("/")[1] || "en");
	let isRefreshing = $state(false);
	let minutesTick = $state(0);

	type KpiCard = {
		label: string;
		value: number;
		change: string;
		context: string;
		trend: "up" | "neutral";
		severity: "good" | "warn";
		sparkline: number[];
		href: string;
	};

	type DonutSlice = {
		label: string;
		value: number;
		strokeClass: string;
		dotClass: string;
		href: string;
	};

	type RecentEvent = {
		id: string;
		eventKey: string;
		eventValues?: Record<string, string>;
		eventTitle: string;
		actor: string;
		actorTitle: string;
		time: string;
		dotClass: string;
		href: string;
	};

	const DONUT_RADIUS = 38;
	const DONUT_CIRCUMFERENCE = 2 * Math.PI * DONUT_RADIUS;

	let greeting = $derived.by(() => {
		const firstName = (data.session?.name ?? $_("admin.dashboard.default_name")).split(" ")[0];
		const hour = new Date().getHours();
		if (hour < 12) return `${$_("greet.morning")}, ${firstName}`;
		if (hour < 18) return `${$_("greet.afternoon")}, ${firstName}`;
		return `${$_("greet.evening")}, ${firstName}`;
	});

	let kpis = $derived<KpiCard[]>(data.kpis);
	let shipmentStatus = $derived<DonutSlice[]>(data.shipmentStatus);

	let totalShipments = $derived(
		shipmentStatus.reduce((sum, status) => sum + status.value, 0),
	);
	let minutesAgo = $derived.by(() => {
		minutesTick;
		if (typeof globalThis.window === "undefined")
			return $_("common.just_now");
		const diff = Date.now() - new Date(data.lastRefresh).getTime();
		const mins = Math.max(0, Math.floor(diff / 60000));
		if (mins === 0) return $_("common.just_now");
		return $_("common.minutes_ago", { values: { minutes: mins } });
	});

	let donutSegments = $derived.by(() => {
		let offset = 0;
		return shipmentStatus.map((status) => {
			const ratio = totalShipments === 0 ? 0 : status.value / totalShipments;
			const dash = ratio * DONUT_CIRCUMFERENCE;
			const segmentOffset = offset;
			offset += dash;
			return {
				...status,
				dash,
				gap: Math.max(DONUT_CIRCUMFERENCE - dash, 0),
				offset: segmentOffset,
			};
		});
	});

	let recentEvents = $derived<RecentEvent[]>(data.recentEvents);

	async function handleRefresh() {
		isRefreshing = true;
		try {
			await invalidateAll();
		} finally {
			isRefreshing = false;
		}
	}

	function sparklinePoints(values: number[]): { line: string; area: string } {
		if (values.length === 0) return { line: "", area: "" };
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min || 1;
		const step = values.length > 1 ? 100 / (values.length - 1) : 0;
		const pts = values.map((v, i) => {
			const x = values.length === 1 ? 50 : i * step;
			const y = 28 - ((v - min) / range) * 24 + 2;
			return { x, y };
		});
		const line = pts.map((pt) => `${pt.x},${pt.y}`).join(" ");
		const first = pts[0];
		const last = pts[pts.length - 1];
		if (!first || !last) return { line: "", area: "" };
		const area =
			pts.length === 1
				? `M${first.x},${first.y} L${first.x},30 L${first.x},30 Z`
				: `M${first.x},${first.y} ${pts
						.slice(1)
						.map((pt) => `L${pt.x},${pt.y}`)
						.join(" ")} L${last.x},30 L${first.x},30 Z`;
		return { line, area };
	}

	if (typeof globalThis.window !== "undefined") {
		const intervalId = window.setInterval(() => {
			minutesTick += 1;
		}, 30000);
		onDestroy(() => window.clearInterval(intervalId));
	}

</script>

<section
	class="stagger stagger-1 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between"
>
	<div>
		<h1 class="text-2xl font-bold text-surface-50">{greeting}</h1>
		<p class="mt-1 text-sm text-surface-400">
			{$_("admin.dashboard.subheadline")}
		</p>
		<div class="mt-2 flex items-center gap-3">
			<span
				class="flex items-center gap-1.5 rounded-full bg-accent/10 px-2 py-0.5 text-[11px] font-semibold text-accent"
			>
				<span class="pulse-dot h-1.5 w-1.5 rounded-full bg-accent"
				></span>
				{$_("admin.dashboard.health_status")}: {$_("admin.dashboard.health_healthy")}
			</span>
			<span class="text-[11px] text-surface-600"
				>{$_("updated")} {minutesAgo}</span
			>
		</div>
	</div>
	<div class="flex items-center gap-2">
		<a
			href={`/${lang}/app/admin/clients/new`}
			class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.dashboard.action.create_client")}
		</a>
		<a
			href={`/${lang}/app/admin/offices/new`}
			class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.dashboard.action.create_office")}
		</a>
		<a
			href={`/${lang}/app/admin/employees/new`}
			class="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-surface-950 transition-colors hover:bg-accent-hover"
		>
			{$_("admin.dashboard.action.create_employee")}
		</a>
		<button
			onclick={handleRefresh}
			aria-label={$_("admin.dashboard.action.refresh")}
			class="rounded-lg bg-surface-800 p-2 text-surface-400 transition-colors hover:bg-surface-700"
		>
			<svg
				class={["h-5 w-5", isRefreshing && "animate-spin"]}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
				/>
			</svg>
		</button>
	</div>
</section>

<section
	class="stagger stagger-2 mt-6 grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4"
>
	{#each kpis as kpi (kpi.label)}
		{@const sp = sparklinePoints(kpi.sparkline)}
		<a
			href={kpi.href}
			class="group relative block cursor-pointer overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900 p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-surface-600/50 hover:shadow-lg hover:shadow-black/20"
		>
			<div class="flex items-center justify-between">
				<span
					class="text-xs font-medium uppercase tracking-wider text-surface-400"
					>{kpi.label}</span
				>
				<span
					class={[
						"h-2 w-2 rounded-full",
						kpi.severity === "good" ? "bg-accent" : "bg-amber-400",
					]}
				></span>
			</div>
			<div class="mt-1 text-3xl font-bold text-surface-50">
				{kpi.value}
			</div>
			<div class="mt-1 flex items-center gap-1.5">
				{#if kpi.trend === "up"}
					<svg
						class="h-3 w-3 text-accent"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M5 15l7-7 7 7"
						/>
					</svg>
					<span class="text-xs font-medium text-accent"
						>{kpi.change}</span
					>
				{:else}
					<svg
						class="h-3 w-3 text-amber-400"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M5 12h14"
						/>
					</svg>
					<span class="text-xs font-medium text-amber-400"
						>{kpi.change}</span
					>
				{/if}
			</div>
			<div class="mt-1 text-[11px] text-surface-600">{kpi.context}</div>
			<svg
				class={[
					"mt-3 h-8 w-full",
					kpi.severity === "good"
						? "text-accent/40"
						: "text-amber-400/40",
				]}
				viewBox="0 0 100 32"
				preserveAspectRatio="none"
			>
				<path d={sp.area} fill="currentColor" />
				<polyline
					points={sp.line}
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			</svg>
			{#if isRefreshing}
				<div
					class="absolute inset-0 animate-pulse rounded-xl bg-surface-800/80"
				></div>
			{/if}
		</a>
	{/each}
</section>

<div class="stagger stagger-3 mt-6 grid grid-cols-1 gap-4 lg:grid-cols-5">
	<div
		class="relative overflow-hidden rounded-xl border border-surface-700/50 bg-surface-900 lg:col-span-2"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<div class="flex items-center justify-between">
				<span class="text-sm font-semibold text-surface-50"
					>{$_("admin.dashboard.shipments_by_status")}</span
				>
				<a
					href={`/${lang}/app/admin/shipments`}
					class="text-xs text-accent transition-colors hover:text-accent-hover"
					>{$_("view_all")}</a
				>
			</div>
		</div>
		<div class="px-5 py-5">
			<div class="mx-auto h-52 w-52">
				<div class="relative flex h-full w-full items-center justify-center">
					<svg class="h-48 w-48" viewBox="0 0 120 120">
						<circle
							cx="60"
							cy="60"
							r={DONUT_RADIUS}
							fill="none"
							stroke="currentColor"
							stroke-width="12"
							class="text-surface-800"
						/>
						{#each donutSegments as segment (segment.label)}
							<circle
								cx="60"
								cy="60"
								r={DONUT_RADIUS}
								fill="none"
								stroke="currentColor"
								stroke-width="12"
								stroke-linecap="round"
								class={segment.strokeClass}
								stroke-dasharray={`${segment.dash} ${segment.gap}`}
								stroke-dashoffset={-segment.offset}
								transform="rotate(-90 60 60)"
							/>
						{/each}
					</svg>
					<div class="absolute text-center">
						<div class="text-2xl font-bold text-surface-50">{totalShipments}</div>
						<div class="text-[11px] uppercase tracking-wider text-surface-600">
							{$_("admin.dashboard.total")}
						</div>
					</div>
				</div>
			</div>
			<div class="mx-auto mt-4 w-full max-w-md space-y-2">
				{#each shipmentStatus as status (status.label)}
					<a
						href={status.href}
						class="group flex cursor-pointer items-center justify-between rounded-lg border border-surface-800 bg-surface-900/40 px-3 py-2 transition-colors hover:bg-surface-800/60"
					>
						<span class="flex items-center gap-2 text-sm text-surface-200">
							<span class={[
								"h-2 w-2 rounded-full",
								status.dotClass,
							]}></span>
							{status.label}
						</span>
						<span class="text-sm font-medium text-surface-400">{status.value}</span>
					</a>
				{/each}
			</div>
		</div>
		{#if isRefreshing}
			<div class="absolute inset-0 animate-pulse bg-surface-800/80"></div>
		{/if}
	</div>

	<div
		class="rounded-xl border border-surface-700/50 bg-surface-900 lg:col-span-3"
	>
		<div class="border-b border-surface-700/50 px-5 py-4">
			<div class="flex items-center justify-between">
				<span class="text-sm font-semibold text-surface-50">{$_("admin.dashboard.recent_events")}</span>
				<a
					href={`/${lang}/app/admin/audit`}
					class="text-xs text-accent transition-colors hover:text-accent-hover"
					>{$_("view_all")}</a
				>
			</div>
		</div>
		<div class="overflow-x-auto">
			<table class="w-full min-w-[420px] table-fixed">
				<thead>
					<tr>
						<th
							class="px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("admin.dashboard.table.event")}
						</th>
						<th
							class="w-40 px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("admin.dashboard.table.actor")}
						</th>
						<th
							class="w-20 px-5 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-surface-600"
						>
							{$_("admin.dashboard.table.time")}
						</th>
					</tr>
				</thead>
				<tbody>
					{#each recentEvents as event (event.id)}
						<tr
							onclick={() => goto(event.href)}
							class="group cursor-pointer border-t border-surface-800 transition-colors hover:bg-surface-800/50"
						>
							<td class="px-5 py-3 text-sm text-surface-200">
								<div class="flex min-w-0 items-center gap-2">
									<span
										class={["h-1.5 w-1.5 shrink-0 rounded-full", event.dotClass]}
									></span>
									<span class="block min-w-0 truncate" title={event.eventTitle}
										>{$_(event.eventKey, { values: event.eventValues })}</span
									>
								</div>
							</td>
							<td class="px-5 py-3 text-sm text-surface-400">
								<span class="block truncate" title={event.actorTitle}
									>{event.actor}</span
								>
							</td>
							<td class="px-5 py-3 text-sm text-surface-400 whitespace-nowrap"
								>{event.time}</td
							>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div>

<style>
	@keyframes fadeSlideUp {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.stagger {
		animation: fadeSlideUp 0.4s ease-out both;
	}
	.stagger-1 {
		animation-delay: 0.05s;
	}
	.stagger-2 {
		animation-delay: 0.1s;
	}
	.stagger-3 {
		animation-delay: 0.2s;
	}

	@keyframes pulse-dot {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.5;
			transform: scale(1.5);
		}
	}

	.pulse-dot {
		animation: pulse-dot 2s ease-in-out infinite;
	}
</style>
