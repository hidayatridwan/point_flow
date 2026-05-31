<script lang="ts">
	import type { StockConversionStatus } from '$lib/api/stock-conversions.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { status }: { status: StockConversionStatus } = $props();

	const labels = $derived<Record<StockConversionStatus, string>>({
		draft: t('inventory.stockConversions.statusDraft'),
		posted: t('inventory.stockConversions.statusPosted'),
		cancelled: t('inventory.stockConversions.statusCancelled')
	});

	const tones: Record<StockConversionStatus, BadgeVariant> = {
		draft: 'neutral',
		posted: 'success',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
