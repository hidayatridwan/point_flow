<script lang="ts">
	import type { PurchaseOrderStatus } from '$lib/api/purchase-orders.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: PurchaseOrderStatus } = $props();

	const labels = $derived<Record<PurchaseOrderStatus, string>>({
		draft: t('procurement.purchaseOrders.statusDraft'),
		ordered: t('procurement.purchaseOrders.statusOrdered'),
		partially_received: t('procurement.purchaseOrders.statusPartiallyReceived'),
		received: t('procurement.purchaseOrders.statusReceived'),
		cancelled: t('procurement.purchaseOrders.statusCancelled')
	});

	const tones: Record<PurchaseOrderStatus, BadgeVariant> = {
		draft: 'neutral',
		ordered: 'info',
		partially_received: 'warning',
		received: 'success',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
