<script lang="ts">
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { OnlineOrder } from '$lib/api/online-orders.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import OnlineOrderDetailDialog from './online-order-detail-dialog.svelte';

	let {
		order,
		onChanged
	}: {
		order: OnlineOrder;
		onChanged?: () => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);

	function openDetail() {
		isDetailOpen = true;
	}
</script>

<div class="flex justify-center">
	<Button
		variant="ghost"
		size="icon-sm"
		class="size-8"
		onclick={openDetail}
		aria-label={`${t('common.detail')}: ${order.order_no}`}
		title={t('common.detail')}
	>
		<EyeIcon />
	</Button>

	<OnlineOrderDetailDialog orderId={order.id} bind:open={isDetailOpen} {onChanged} />
</div>
