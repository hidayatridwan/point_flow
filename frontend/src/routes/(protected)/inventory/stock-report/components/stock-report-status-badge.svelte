<script lang="ts">
	import type { StockReportItem } from '$lib/api/stock-report.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { item }: { item: StockReportItem } = $props();

	const status = $derived(getStockStatus(item));

	function getStockStatus(item: StockReportItem): { label: string; tone: BadgeVariant } {
		if (!item.is_active) {
			return { label: 'Inactive', tone: 'neutral' };
		}

		if (item.is_low_stock) {
			return { label: 'Low Stock', tone: 'destructive' };
		}

		return { label: 'In Stock', tone: 'success' };
	}
</script>

<Badge variant={status.tone}>{status.label}</Badge>
