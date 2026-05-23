<script lang="ts">
	import type { StockReportItem } from '$lib/api/stock-report.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { item }: { item: StockReportItem } = $props();

	const status = $derived(getStockStatus(item));

	function getStockStatus(item: StockReportItem) {
		if (!item.is_active) {
			return {
				label: 'Inactive',
				class: 'border-zinc-500/20 bg-zinc-50 text-zinc-600 dark:bg-zinc-950/30 dark:text-zinc-300'
			};
		}

		if (item.is_low_stock) {
			return {
				label: 'Low Stock',
				class: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
			};
		}

		return {
			label: 'In Stock',
			class:
				'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300'
		};
	}
</script>

<Badge variant="outline" class={status.class}>{status.label}</Badge>
