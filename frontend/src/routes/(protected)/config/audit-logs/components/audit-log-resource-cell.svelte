<script lang="ts">
	let {
		resourceType,
		resourceId,
		resourceLabel
	}: {
		resourceType: string;
		resourceId: number | null;
		resourceLabel: string | null;
	} = $props();
</script>

<div class="grid min-w-36 gap-0.5">
	<div class="truncate font-medium">{resourceType}</div>
	<div class="truncate text-xs text-muted-foreground">
		{resourceLabel || (resourceId ? `ID ${resourceId}` : '-')}
	</div>
</div>
