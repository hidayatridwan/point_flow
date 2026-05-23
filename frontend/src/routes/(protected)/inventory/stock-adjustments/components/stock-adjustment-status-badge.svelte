<script lang="ts">
	import type { StockAdjustmentStatus } from '$lib/api/stock-adjustments.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { status }: { status: StockAdjustmentStatus } = $props();

	const labels: Record<StockAdjustmentStatus, string> = {
		draft: 'Draft',
		posted: 'Posted',
		cancelled: 'Cancelled'
	};

	const classes: Record<StockAdjustmentStatus, string> = {
		draft: 'border-zinc-500/20 bg-zinc-50 text-zinc-600 dark:bg-zinc-950/30 dark:text-zinc-300',
		posted:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cancelled: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[status]}>{labels[status]}</Badge>
