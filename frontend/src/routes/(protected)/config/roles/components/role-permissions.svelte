<script lang="ts">
	import { Badge } from '$lib/components/ui/badge/index.js';

	let {
		permissions,
		limit = 4
	}: {
		permissions: string[];
		limit?: number;
	} = $props();

	const visiblePermissions = $derived(permissions.slice(0, limit));
	const hiddenCount = $derived(Math.max(0, permissions.length - visiblePermissions.length));
</script>

<div class="flex max-w-96 flex-wrap gap-1.5">
	{#if permissions.length === 0}
		<span class="text-sm text-muted-foreground">No permissions</span>
	{:else}
		{#each visiblePermissions as permission (permission)}
			<Badge variant="outline" class="font-mono text-xs">{permission}</Badge>
		{/each}
		{#if hiddenCount > 0}
			<Badge variant="secondary" class="text-xs">+{hiddenCount}</Badge>
		{/if}
	{/if}
</div>
