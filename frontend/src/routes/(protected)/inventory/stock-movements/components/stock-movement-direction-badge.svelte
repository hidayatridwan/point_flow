<script lang="ts">
	import type { StockMovementDirection } from '$lib/api/stock-movements.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { direction }: { direction: StockMovementDirection } = $props();

	const labels: Record<StockMovementDirection, string> = {
		in: 'In',
		out: 'Out'
	};

	const classes: Record<StockMovementDirection, string> = {
		in: 'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		out: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[direction]}>{labels[direction]}</Badge>
