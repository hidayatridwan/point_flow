<script lang="ts">
	import type { StockMovementType } from '$lib/api/stock-movements.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { type }: { type: StockMovementType } = $props();

	const labels = $derived({
		stock_in: t('inventory.stockMovements.typeStockIn'),
		stock_out: t('inventory.stockMovements.typeStockOut'),
		sale: t('inventory.stockMovements.typeSale'),
		purchase_receipt: t('inventory.stockMovements.typePurchaseReceipt'),
		adjustment: t('inventory.stockMovements.typeAdjustment'),
		conversion_in: t('inventory.stockMovements.typeConversionIn'),
		conversion_out: t('inventory.stockMovements.typeConversionOut'),
		correction: t('inventory.stockMovements.typeCorrection')
	} as Record<string, string>);

	const tones: Record<StockMovementType, BadgeVariant> = {
		stock_in: 'success',
		stock_out: 'destructive',
		sale: 'info',
		purchase_receipt: 'success',
		adjustment: 'warning',
		conversion_in: 'info',
		conversion_out: 'warning',
		correction: 'accent'
	};
</script>

<Badge variant={tones[type]}>{labels[type]}</Badge>
