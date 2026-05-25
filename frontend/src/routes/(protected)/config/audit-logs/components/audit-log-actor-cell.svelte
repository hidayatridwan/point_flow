<script lang="ts">
	let {
		name,
		email,
		actorId
	}: {
		name: string | null;
		email: string | null;
		actorId: number | null;
	} = $props();
</script>

<div class="grid min-w-40 gap-0.5">
	<div class="truncate font-medium">{name || 'System'}</div>
	<div class="truncate text-xs text-muted-foreground">
		{email || (actorId ? `Actor #${actorId}` : 'No actor')}
	</div>
</div>
