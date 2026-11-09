<script lang="ts">
	import BadgeCheckIcon from '@lucide/svelte/icons/badge-check';
	import BanIcon from '@lucide/svelte/icons/ban';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import MessageCircleIcon from '@lucide/svelte/icons/message-circle';
	import PackageCheckIcon from '@lucide/svelte/icons/package-check';
	import PackageIcon from '@lucide/svelte/icons/package';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import { getMember } from '$lib/api/members.js';
	import { getOnlineOrder, type OnlineOrderDetail } from '$lib/api/online-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import OnlineOrderCancelDialog from './online-order-cancel-dialog.svelte';
	import OnlineOrderConfirmDialog from './online-order-confirm-dialog.svelte';
	import OnlineOrderStatusBadge from './online-order-status-badge.svelte';

	let {
		orderId,
		open = $bindable(false),
		onChanged
	}: {
		orderId: number;
		open: boolean;
		onChanged?: () => void | Promise<void>;
	} = $props();

	let detail = $state<OnlineOrderDetail | null>(null);
	let memberLabel = $state<string | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;
	let confirmAction = $state<'pack' | 'ready' | 'deliver' | 'complete' | 'mark-paid' | null>(null);
	let isConfirmOpen = $state(false);
	let isCancelOpen = $state(false);

	$effect(() => {
		if (!open) return;
		void loadDetail(orderId);
	});

	const order = $derived(detail?.order ?? null);

	const primaryAction = $derived.by(() => {
		if (!order) return null;
		if (order.status === 'new')
			return {
				action: 'pack',
				label: t('point_flow.onlineOrders.actionPack'),
				icon: PackageIcon
			} as const;
		if (order.status === 'packing' && order.fulfillment_method === 'pickup')
			return {
				action: 'ready',
				label: t('point_flow.onlineOrders.actionReady'),
				icon: PackageCheckIcon
			} as const;
		if (order.status === 'packing' && order.fulfillment_method === 'delivery')
			return {
				action: 'deliver',
				label: t('point_flow.onlineOrders.actionDeliver'),
				icon: TruckIcon
			} as const;
		if (order.status === 'ready' || order.status === 'delivering')
			return {
				action: 'complete',
				label: t('point_flow.onlineOrders.actionComplete'),
				icon: CircleCheckIcon
			} as const;
		return null;
	});

	const canMarkPaid = $derived(
		order !== null &&
			order.payment_status !== 'paid' &&
			order.status !== 'completed' &&
			order.status !== 'cancelled'
	);

	const canCancel = $derived(order?.status === 'new' || order?.status === 'packing');

	const deliverBlocked = $derived(
		order?.status === 'packing' &&
			order?.fulfillment_method === 'delivery' &&
			order?.payment_method === 'transfer' &&
			order?.payment_status !== 'paid'
	);

	const waHref = $derived.by(() => {
		if (!order) return '#';
		const digits = order.customer_phone.replace(/\D/g, '');
		const normalized = digits.startsWith('0') ? `62${digits.slice(1)}` : digits;
		const text = t('point_flow.onlineOrders.waPrefill').replace('{orderNo}', order.order_no);
		return `https://wa.me/${normalized}?text=${encodeURIComponent(text)}`;
	});

	async function loadDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getOnlineOrder(id);
			if (requestId !== latestRequestId) return;
			detail = result;
			memberLabel = await resolveMemberLabel(result.order.member_id, requestId);
		} catch (error) {
			if (requestId !== latestRequestId) return;
			detail = null;
			memberLabel = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function openConfirm(action: NonNullable<typeof confirmAction>) {
		confirmAction = action;
		isConfirmOpen = true;
	}

	async function handleActionDone(next: OnlineOrderDetail) {
		detail = next;
		memberLabel = await resolveMemberLabel(next.order.member_id, latestRequestId);
		await onChanged?.();
	}

	async function resolveMemberLabel(memberId: number | null, requestId: number) {
		if (!memberId) return null;
		try {
			const member = await getMember(memberId);
			if (requestId !== latestRequestId) return null;
			return `${member.full_name} (${member.member_no})`;
		} catch {
			return null;
		}
	}

	function fulfillmentLabel(value: string) {
		return value === 'delivery'
			? t('point_flow.onlineOrders.fulfillmentDelivery')
			: t('point_flow.onlineOrders.fulfillmentPickup');
	}

	function paymentLabel(value: string) {
		return value === 'transfer'
			? t('point_flow.onlineOrders.paymentTransfer')
			: t('point_flow.onlineOrders.paymentPayAtStore');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('point_flow.onlineOrders.errorLoadDetail');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
		<Dialog.Header>
			<Dialog.Title>{t('point_flow.onlineOrders.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('point_flow.onlineOrders.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-32 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if order && detail}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('point_flow.onlineOrders.colOrderNo')} value={order.order_no} />
					<DetailField label={t('point_flow.onlineOrders.colStatus')}>
						<OnlineOrderStatusBadge status={order.status} />
					</DetailField>
					<DetailField
						label={t('point_flow.onlineOrders.colPayment')}
						value={`${paymentLabel(order.payment_method)} · ${
							order.payment_status === 'paid'
								? t('point_flow.onlineOrders.paymentPaid')
								: t('point_flow.onlineOrders.paymentUnpaid')
						}`}
					/>
					<DetailField
						label={t('point_flow.onlineOrders.customerName')}
						value={order.customer_name}
					/>
					<DetailField
						label={t('point_flow.onlineOrders.customerPhone')}
						value={order.customer_phone}
					/>
					<DetailField label={t('point_flow.onlineOrders.member')} value={memberLabel || '-'} />
					<DetailField
						label={t('point_flow.onlineOrders.colFulfillment')}
						value={fulfillmentLabel(order.fulfillment_method)}
					/>
				</div>

				{#if order.fulfillment_method === 'delivery'}
					<DetailField
						label={t('point_flow.onlineOrders.deliveryAddress')}
						value={order.delivery_address || '-'}
					/>
				{/if}

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('point_flow.onlineOrders.notes')} value={order.notes || '-'} />
					<DetailField
						label={t('point_flow.onlineOrders.cancelReasonField')}
						value={order.cancel_reason || '-'}
					/>
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>{t('point_flow.onlineOrders.itemProduct')}</Table.Head>
									<Table.Head>{t('point_flow.onlineOrders.itemSku')}</Table.Head>
									<Table.Head>{t('point_flow.onlineOrders.itemUnit')}</Table.Head>
									<Table.Head>{t('point_flow.onlineOrders.itemQty')}</Table.Head>
									<Table.Head>{t('point_flow.onlineOrders.itemPrice')}</Table.Head>
									<Table.Head>{t('point_flow.onlineOrders.itemTotal')}</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each detail.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity)}</Table.Cell>
										<Table.Cell>{formatIdr(item.unit_price)}</Table.Cell>
										<Table.Cell>{formatIdr(item.line_total)}</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
					<DetailField
						label={t('point_flow.onlineOrders.subtotal')}
						value={formatIdr(order.subtotal_amount)}
					/>
					<DetailField
						label={t('point_flow.onlineOrders.deliveryFee')}
						value={formatIdr(order.delivery_fee)}
					/>
					<DetailField
						label={t('point_flow.onlineOrders.total')}
						value={formatIdr(order.total_amount)}
					/>
				</div>

				{#if deliverBlocked}
					<ErrorAlert
						title={t('point_flow.onlineOrders.deliverBlockedTitle')}
						message={t('point_flow.onlineOrders.deliverBlockedDesc')}
					/>
				{/if}

				<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap">
					<Button variant="outline" href={waHref} target="_blank" rel="noreferrer">
						<MessageCircleIcon />
						{t('point_flow.onlineOrders.chatWhatsapp')}
					</Button>
					{#if primaryAction}
						<Button
							onclick={() => openConfirm(primaryAction.action)}
							disabled={primaryAction.action === 'deliver' && deliverBlocked}
						>
							<primaryAction.icon />
							{primaryAction.label}
						</Button>
					{/if}
					{#if canMarkPaid}
						<Button variant="secondary" onclick={() => openConfirm('mark-paid')}>
							<BadgeCheckIcon />
							{t('point_flow.onlineOrders.actionMarkPaid')}
						</Button>
					{/if}
					{#if canCancel}
						<Button
							variant="destructive"
							onclick={() => {
								isCancelOpen = true;
							}}
						>
							<BanIcon />
							{t('point_flow.onlineOrders.cancelBtn')}
						</Button>
					{/if}
				</div>
				{#if isLoading}
					<div class="flex items-center gap-2 text-sm text-muted-foreground">
						<Spinner />
						{t('common.ordering')}
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>

{#if order && confirmAction}
	<OnlineOrderConfirmDialog
		{order}
		action={confirmAction}
		bind:open={isConfirmOpen}
		onDone={handleActionDone}
	/>
{/if}
{#if order}
	<OnlineOrderCancelDialog {order} bind:open={isCancelOpen} onDone={handleActionDone} />
{/if}
