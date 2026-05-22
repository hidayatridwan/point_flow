<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		description,
		actions
	}: {
		title: string;
		description?: string;
		actions?: Snippet;
	} = $props();
</script>

<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
	<div class="flex flex-col gap-1">
		<h1 class="text-xl font-semibold tracking-tight">{title}</h1>
		{#if description}
			<p class="max-w-2xl text-sm text-muted-foreground">{description}</p>
		{/if}
	</div>

	{#if actions}
		<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row sm:items-center">
			{@render actions()}
		</div>
	{/if}
</div>
