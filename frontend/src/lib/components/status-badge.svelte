<script lang="ts">
	import { Badge } from '$lib/components/ui/badge/index.js';

	let {
		active,
		activeLabel = 'Active',
		inactiveLabel = 'Inactive'
	}: {
		active: boolean;
		activeLabel?: string;
		inactiveLabel?: string;
	} = $props();
</script>

<Badge variant={active ? 'default' : 'ghost'}>
	{active ? activeLabel : inactiveLabel}
</Badge>
