<script lang="ts">
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		permissions,
		limit = 3
	}: {
		permissions: string[];
		limit?: number;
	} = $props();

	const visiblePermissions = $derived(permissions.slice(0, limit));
	const hiddenCount = $derived(Math.max(0, permissions.length - visiblePermissions.length));
</script>

<div class="flex max-w-80 flex-wrap gap-1.5">
	{#if permissions.length === 0}
		<span class="text-sm text-muted-foreground">{t('config.users.noPermissions')}</span>
	{:else}
		{#each visiblePermissions as permission (permission)}
			<Badge variant="outline" class="font-mono text-xs">{permission}</Badge>
		{/each}
		{#if hiddenCount > 0}
			<Badge variant="secondary" class="text-xs">+{hiddenCount}</Badge>
		{/if}
	{/if}
</div>
