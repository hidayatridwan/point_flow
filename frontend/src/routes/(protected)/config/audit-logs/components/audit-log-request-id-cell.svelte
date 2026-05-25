<script lang="ts">
	let { requestId }: { requestId: string | null } = $props();
</script>

<div class="max-w-44 truncate font-mono text-xs" title={requestId || '-'}>
	{requestId || '-'}
</div>
