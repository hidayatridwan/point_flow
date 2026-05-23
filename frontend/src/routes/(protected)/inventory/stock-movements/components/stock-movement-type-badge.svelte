<script lang="ts">
	import type { StockMovementType } from '$lib/api/stock-movements.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { type }: { type: StockMovementType } = $props();

	const labels: Record<StockMovementType, string> = {
		stock_in: 'Stock In',
		stock_out: 'Stock Out',
		sale: 'Sale',
		purchase_receipt: 'Purchase Receipt',
		adjustment: 'Adjustment',
		conversion_in: 'Conversion In',
		conversion_out: 'Conversion Out',
		correction: 'Correction'
	};

	const classes: Record<StockMovementType, string> = {
		stock_in:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		stock_out: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300',
		sale: 'border-blue-600/20 bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-300',
		purchase_receipt:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		adjustment:
			'border-amber-600/20 bg-amber-50 text-amber-700 dark:bg-amber-950/30 dark:text-amber-300',
		conversion_in:
			'border-cyan-600/20 bg-cyan-50 text-cyan-700 dark:bg-cyan-950/30 dark:text-cyan-300',
		conversion_out:
			'border-orange-600/20 bg-orange-50 text-orange-700 dark:bg-orange-950/30 dark:text-orange-300',
		correction:
			'border-violet-600/20 bg-violet-50 text-violet-700 dark:bg-violet-950/30 dark:text-violet-300'
	};
</script>

<Badge variant="outline" class={classes[type]}>{labels[type]}</Badge>
