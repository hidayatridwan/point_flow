<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		label,
		value,
		children
	}: {
		label: string;
		value?: string | number | null;
		children?: Snippet;
	} = $props();
</script>

<div class="rounded-md border bg-muted/20 p-3">
	<p class="text-xs font-medium text-muted-foreground">{label}</p>
	{#if children}
		<div class="mt-2">
			{@render children()}
		</div>
	{:else}
		<p class="mt-1 font-medium">{value ?? '-'}</p>
	{/if}
</div>
