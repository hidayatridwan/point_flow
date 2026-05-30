<script lang="ts">
	import type { PurchaseOrderStatus } from '$lib/api/purchase-orders.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: PurchaseOrderStatus } = $props();

	const labels = $derived<Record<PurchaseOrderStatus, string>>({
		draft: t('procurement.purchaseOrders.statusDraft'),
		ordered: t('procurement.purchaseOrders.statusOrdered'),
		partially_received: t('procurement.purchaseOrders.statusPartiallyReceived'),
		received: t('procurement.purchaseOrders.statusReceived'),
		cancelled: t('procurement.purchaseOrders.statusCancelled')
	});

	const classes: Record<PurchaseOrderStatus, string> = {
		draft: 'border-zinc-500/20 bg-zinc-50 text-zinc-600 dark:bg-zinc-950/30 dark:text-zinc-300',
		ordered: 'border-sky-600/20 bg-sky-50 text-sky-700 dark:bg-sky-950/30 dark:text-sky-300',
		partially_received:
			'border-amber-600/20 bg-amber-50 text-amber-700 dark:bg-amber-950/30 dark:text-amber-300',
		received:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cancelled: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[status]}>{labels[status]}</Badge>
