<script lang="ts">
	import Undo2Icon from '@lucide/svelte/icons/undo-2';
	import {
		createSaleReturn,
		getReturnableSaleItems,
		type ReturnableSaleItem,
		type Sale,
		type SaleReturnDetail
	} from '$lib/api/sales.js';
	import { getStoredActiveCashRegisterSessionId } from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	type ReturnFormItem = {
		item: ReturnableSaleItem;
		quantity: string;
		restock: boolean;
		reason: string;
	};

	let {
		sale,
		open = $bindable(false),
		onReturned
	}: {
		sale: Sale;
		open: boolean;
		onReturned?: (saleReturn: SaleReturnDetail) => void | Promise<void>;
	} = $props();

	let formItems = $state<ReturnFormItem[]>([]);
	let reason = $state('');
	let notes = $state('');
	let activeSessionId = $state<number | null>(null);
	let isLoading = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	const totalRefund = $derived(
		formItems.reduce((total, item) => total + getQuantity(item) * item.item.unit_price, 0)
	);
	const selectedItemCount = $derived(
		formItems.filter((item) => getQuantity(item) > 0).length
	);
	const hasReturnableItems = $derived(
		formItems.some((item) => item.item.returnable_quantity > 0)
	);
	const canSubmit = $derived(
		Boolean(activeSessionId) && selectedItemCount > 0 && !isLoading && !isSubmitting
	);

	$effect(() => {
		if (!open) return;
		reason = '';
		notes = '';
		errorMessage = null;
		activeSessionId = getStoredActiveCashRegisterSessionId();
		void loadReturnableItems();
	});

	async function loadReturnableItems() {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;
		formItems = [];

		try {
			const items = await getReturnableSaleItems(sale.id);
			if (requestId !== latestRequestId) return;
			formItems = items.map((item) => ({
				item,
				quantity: '',
				restock: true,
				reason: ''
			}));
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error, 'Failed to load returnable items.');
		} finally {
			if (requestId === latestRequestId) isLoading = false;
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		const validationError = getValidationError();
		if (validationError) {
			errorMessage = validationError;
			return;
		}

		isSubmitting = true;
		errorMessage = null;

		try {
			const saleReturn = await createSaleReturn({
				sale_id: sale.id,
				cash_register_session_id: activeSessionId as number,
				reason: emptyToNull(reason),
				notes: emptyToNull(notes),
				items: formItems
					.filter((item) => getQuantity(item) > 0)
					.map((item) => ({
						sale_item_id: item.item.id,
						quantity: getQuantity(item),
						restock: item.restock,
						reason: emptyToNull(item.reason)
					}))
			});

			await onReturned?.(saleReturn);
			toast.success(t('pos.sales.return.createBtn'), {
				description: `${saleReturn.return_no} refund ${formatIdr(saleReturn.refund_amount)}.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('pos.sales.return.returnFailed'));
		} finally {
			isSubmitting = false;
		}
	}

	function fillMax(item: ReturnFormItem) {
		item.quantity = formatQuantityInput(item.item.returnable_quantity);
	}

	function clearQuantity(item: ReturnFormItem) {
		item.quantity = '';
	}

	function getValidationError() {
		if (!activeSessionId) return t('pos.sales.return.validNoRegister');
		if (!selectedItemCount) return t('pos.sales.return.validNoItems');

		for (const item of formItems) {
			const quantity = getQuantity(item);
			if (quantity < 0) return `${item.item.product_name} return quantity cannot be negative.`;
			if (quantity - item.item.returnable_quantity > 0.000001) {
				return `${item.item.product_name} return quantity exceeds the remaining returnable quantity.`;
			}
		}

		return null;
	}

	function getQuantity(item: ReturnFormItem) {
		const value = Number(item.quantity);
		return Number.isFinite(value) ? value : 0;
	}

	function formatQuantityInput(value: number) {
		return Number.isInteger(value) ? String(value) : String(value);
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto p-0 sm:max-w-5xl">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('pos.sales.return.title')}</Dialog.Title>
				<Dialog.Description>
					{t('pos.sales.return.desc').replace('{txNo}', sale.transaction_no)}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-5 px-6">
				<div class="grid grid-cols-1 gap-3 text-sm sm:grid-cols-3">
					<div>
						<div class="text-muted-foreground">{t('pos.sales.return.tx')}</div>
						<div class="font-medium">{sale.transaction_no}</div>
					</div>
					<div>
						<div class="text-muted-foreground">{t('pos.sales.return.register')}</div>
						<div class="font-medium">{sale.cash_register_code ?? '-'}</div>
					</div>
					<div>
						<div class="text-muted-foreground">{t('pos.sales.return.refundTotal')}</div>
						<div class="font-semibold">{formatIdr(totalRefund)}</div>
					</div>
				</div>

				{#if !activeSessionId}
					<div class="rounded-md border border-destructive/30 bg-destructive/5 p-4">
						<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
							<p class="text-sm text-destructive">
								{t('pos.sales.return.noRegister')}
							</p>
							<Button
								type="button"
								variant="outline"
								size="sm"
								href="/pos/register-session"
							>
								{t('pos.sales.return.goRegisterBtn')}
							</Button>
						</div>
					</div>
				{/if}

				{#if isLoading}
					<div class="flex h-32 items-center justify-center gap-2 text-sm text-muted-foreground">
						<Spinner />
						{t('pos.sales.return.loading')}
					</div>
				{:else if formItems.length}
					<div class="overflow-hidden rounded-md border">
						<div class="overflow-x-auto">
							<Table.Root>
								<Table.Header>
									<Table.Row>
										<Table.Head class="min-w-52">{t('pos.sales.return.colProduct')}</Table.Head>
										<Table.Head class="w-24">{t('pos.sales.return.colSold')}</Table.Head>
										<Table.Head class="w-28">{t('pos.sales.return.colReturned')}</Table.Head>
										<Table.Head class="w-32">{t('pos.sales.return.colAvailable')}</Table.Head>
										<Table.Head class="w-36">{t('pos.sales.return.colReturnQty')}</Table.Head>
										<Table.Head class="w-28">{t('pos.sales.return.colRestock')}</Table.Head>
										<Table.Head class="min-w-44">{t('pos.sales.return.colItemReason')}</Table.Head>
										<Table.Head class="w-32">{t('pos.sales.return.colRefund')}</Table.Head>
									</Table.Row>
								</Table.Header>
								<Table.Body>
									{#each formItems as item (item.item.id)}
										<Table.Row class={item.item.returnable_quantity <= 0 ? 'opacity-60' : undefined}>
											<Table.Cell>
												<div class="font-medium">{item.item.product_name}</div>
												<div class="text-xs text-muted-foreground">
													{item.item.product_sku} / {item.item.unit}
												</div>
											</Table.Cell>
											<Table.Cell>{formatNumber(item.item.quantity)}</Table.Cell>
											<Table.Cell>{formatNumber(item.item.returned_quantity)}</Table.Cell>
											<Table.Cell>{formatNumber(item.item.returnable_quantity)}</Table.Cell>
											<Table.Cell>
												<div class="flex min-w-32 items-center gap-2">
													<Input
														type="number"
														min="0"
														step="0.001"
														max={item.item.returnable_quantity}
														class="h-9 w-24"
														disabled={isSubmitting || item.item.returnable_quantity <= 0}
														bind:value={item.quantity}
													/>
													<Button
														type="button"
														variant="outline"
														size="sm"
														class="h-9 px-2"
														disabled={isSubmitting || item.item.returnable_quantity <= 0}
														onclick={() => fillMax(item)}
													>
														Max
													</Button>
												</div>
											</Table.Cell>
											<Table.Cell>
												<label class="flex items-center gap-2 text-sm">
													<Checkbox
														disabled={isSubmitting || item.item.returnable_quantity <= 0}
														bind:checked={item.restock}
													/>
													<span>{t('pos.sales.return.yesLabel')}</span>
												</label>
											</Table.Cell>
											<Table.Cell>
												<Input
													class="h-9 min-w-40"
													placeholder={t('pos.sales.return.optional')}
													disabled={isSubmitting || item.item.returnable_quantity <= 0}
													bind:value={item.reason}
												/>
											</Table.Cell>
											<Table.Cell>
												<div class="flex items-center justify-between gap-2">
													<span>{formatIdr(getQuantity(item) * item.item.unit_price)}</span>
													{#if getQuantity(item) > 0}
														<Button
															type="button"
															variant="ghost"
															size="sm"
															class="h-8 px-2"
															disabled={isSubmitting}
															onclick={() => clearQuantity(item)}
														>
															Clear
														</Button>
													{/if}
												</div>
											</Table.Cell>
										</Table.Row>
									{/each}
								</Table.Body>
							</Table.Root>
						</div>
					</div>

					{#if !hasReturnableItems}
						<p class="text-sm text-muted-foreground">
							{t('pos.sales.return.allReturned')}
						</p>
					{/if}
				{:else}
					<p class="text-sm text-muted-foreground">{t('pos.sales.return.noItems')}</p>
				{/if}

				<div class="grid gap-4 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for={`sale-return-reason-${sale.id}`}>Return Reason</Label>
						<Input
							id={`sale-return-reason-${sale.id}`}
							placeholder={t('pos.sales.return.reasonPlaceholder')}
							disabled={isSubmitting}
							bind:value={reason}
						/>
					</div>
					<div class="space-y-2">
						<Label for={`sale-return-notes-${sale.id}`}>Notes</Label>
						<Textarea
							id={`sale-return-notes-${sale.id}`}
							placeholder={t('pos.sales.return.notesPlaceholder')}
							class="min-h-10"
							disabled={isSubmitting}
							bind:value={notes}
						/>
					</div>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('pos.sales.return.returnFailed')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('pos.sales.return.cancelBtn')}
					</Dialog.Close>
				<Button type="submit" disabled={!canSubmit}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<Undo2Icon />
					{/if}
					{isSubmitting ? t('pos.sales.return.creating') : t('pos.sales.return.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
