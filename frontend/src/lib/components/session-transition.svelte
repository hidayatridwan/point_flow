<script lang="ts">
	import LoaderCircleIcon from '@lucide/svelte/icons/loader-circle';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils.js';

	let {
		class: className,
		title = 'Processing...',
		description = 'Please wait a moment.',
		...restProps
	}: HTMLAttributes<HTMLDivElement> & {
		title?: string;
		description?: string | null;
	} = $props();
</script>

<div
	class={cn('flex min-h-80 flex-col items-center justify-center gap-3 text-center', className)}
	{...restProps}
>
	<div class="flex size-12 items-center justify-center rounded-full border bg-background shadow-sm">
		<LoaderCircleIcon class="size-5 animate-spin text-primary" />
	</div>
	<div class="space-y-1">
		<p class="text-sm font-medium">{title}</p>
		{#if description}
			<p class="text-sm text-muted-foreground">{description}</p>
		{/if}
	</div>
</div>
