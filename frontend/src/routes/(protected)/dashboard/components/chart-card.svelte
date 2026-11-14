<script lang="ts">
	import type { Snippet } from 'svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		title,
		description,
		loading = false,
		error = null,
		empty = false,
		emptyText,
		action,
		children
	}: {
		title: string;
		description?: string;
		loading?: boolean;
		error?: string | null;
		empty?: boolean;
		emptyText?: string;
		action?: Snippet;
		children: Snippet;
	} = $props();
</script>

<!-- Flat card: hairline border, no resting shadow, roomier padding. -->
<Card.Root
	class="gap-4 border border-border/70 shadow-none ring-0 transition-shadow duration-200 [--card-spacing:--spacing(5)] focus-within:shadow-sm hover:shadow-sm"
>
	<Card.Header class="flex flex-row items-start justify-between gap-3">
		<div class="space-y-1">
			<Card.Title class="text-base font-semibold">{title}</Card.Title>
			{#if description}
				<Card.Description>{description}</Card.Description>
			{/if}
		</div>
		{#if action}
			<div class="shrink-0">{@render action()}</div>
		{/if}
	</Card.Header>
	<Card.Content>
		{#if loading}
			<Skeleton class="h-[260px] w-full rounded-md" />
		{:else if error}
			<div
				class="flex h-[260px] items-center justify-center rounded-md border border-dashed text-sm text-destructive"
			>
				{error}
			</div>
		{:else if empty}
			<div
				class="flex h-[260px] items-center justify-center rounded-md border border-dashed text-sm text-muted-foreground"
			>
				{emptyText ?? t('dashboard.charts.empty')}
			</div>
		{:else}
			{@render children()}
		{/if}
	</Card.Content>
</Card.Root>
