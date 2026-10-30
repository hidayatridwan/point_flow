<script lang="ts">
	import type { GoodsReceiptStatus } from '$lib/api/goods-receipts.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { status }: { status: GoodsReceiptStatus } = $props();

	const labels = $derived<Record<GoodsReceiptStatus, string>>({
		draft: t('procurement.goodsReceipts.statusDraft'),
		posted: t('procurement.goodsReceipts.statusPosted'),
		cancelled: t('procurement.goodsReceipts.statusCancelled')
	});

	const tones: Record<GoodsReceiptStatus, BadgeVariant> = {
		draft: 'ghost',
		posted: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
