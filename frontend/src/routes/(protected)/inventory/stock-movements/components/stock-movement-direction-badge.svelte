<script lang="ts">
	import type { StockMovementDirection } from '$lib/api/stock-movements.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { direction }: { direction: StockMovementDirection } = $props();

	const labels = $derived({
		in: t('inventory.stockMovements.directionIn'),
		out: t('inventory.stockMovements.directionOut')
	} as Record<string, string>);

	const tones: Record<StockMovementDirection, BadgeVariant> = {
		in: 'success',
		out: 'destructive'
	};
</script>

<Badge variant={tones[direction]}>{labels[direction]}</Badge>
