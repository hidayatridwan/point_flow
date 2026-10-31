<script lang="ts">
	import type { StockReportItem } from '$lib/api/stock-report.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { item }: { item: StockReportItem } = $props();

	const status = $derived(getStockStatus(item));

	function getStockStatus(item: StockReportItem): { label: string; tone: BadgeVariant } {
		if (!item.is_active) {
			return { label: t('inventory.stockReport.statusInactive'), tone: 'ghost' };
		}

		if (item.is_low_stock) {
			return { label: t('inventory.stockReport.statusLowStock'), tone: 'destructive' };
		}

		return { label: t('inventory.stockReport.statusInStock'), tone: 'default' };
	}
</script>

<Badge variant={status.tone}>{status.label}</Badge>
