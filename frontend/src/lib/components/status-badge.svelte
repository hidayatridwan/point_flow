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

<Badge
	variant="outline"
	class={active
		? 'border-emerald-600/20 bg-emerald-50 text-emerald-700'
		: 'border-zinc-500/20 bg-zinc-50 text-zinc-600'}
>
	{active ? activeLabel : inactiveLabel}
</Badge>
