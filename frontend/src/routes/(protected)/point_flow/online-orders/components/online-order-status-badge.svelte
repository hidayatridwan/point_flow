<script lang="ts">
	import type { OnlineOrderStatus } from '$lib/api/online-orders.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: OnlineOrderStatus } = $props();

	const labels = $derived<Record<OnlineOrderStatus, string>>({
		new: t('point_flow.onlineOrders.statusNew'),
		packing: t('point_flow.onlineOrders.statusPacking'),
		ready: t('point_flow.onlineOrders.statusReady'),
		delivering: t('point_flow.onlineOrders.statusDelivering'),
		completed: t('point_flow.onlineOrders.statusCompleted'),
		cancelled: t('point_flow.onlineOrders.statusCancelled')
	});

	const tones: Record<OnlineOrderStatus, BadgeVariant> = {
		new: 'default',
		packing: 'secondary',
		ready: 'secondary',
		delivering: 'default',
		completed: 'outline',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
