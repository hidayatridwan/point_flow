<script lang="ts">
	import type { StockAdjustmentStatus } from '$lib/api/stock-adjustments.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { status }: { status: StockAdjustmentStatus } = $props();

	const labels = $derived<Record<StockAdjustmentStatus, string>>({
		draft: t('inventory.stockAdjustments.statusDraft'),
		posted: t('inventory.stockAdjustments.statusPosted'),
		cancelled: t('inventory.stockAdjustments.statusCancelled')
	});

	const tones: Record<StockAdjustmentStatus, BadgeVariant> = {
		draft: 'neutral',
		posted: 'success',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
