<script lang="ts">
	import PrinterIcon from '@lucide/svelte/icons/printer';
	import { getSale, type SaleDetail } from '$lib/api/sales.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { printSaleReceipt } from '$lib/receipt/sale-receipt.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';

	let {
		saleId,
		saleDetail = null,
		open = $bindable(false)
	}: {
		saleId?: number;
		saleDetail?: SaleDetail | null;
		open: boolean;
	} = $props();

	let sale = $state<SaleDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (saleDetail) {
			sale = saleDetail;
		}
	});

	$effect(() => {
		if (!open) return;
		if (saleDetail) {
			sale = saleDetail;
			errorMessage = null;
			return;
		}
		if (!saleId) return;
		void loadSaleDetail(saleId);
	});

	async function loadSaleDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getSale(id);
			if (requestId !== latestRequestId) return;
			sale = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			sale = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function handlePrint() {
		if (!sale) return;

		try {
			printSaleReceipt(sale);
			toast.success('Receipt print window opened.', {
				description: sale.transaction_no
			});
		} catch (error) {
			errorMessage = getErrorMessage(error);
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to print receipt.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto p-0 sm:max-w-lg">
		<div class="grid gap-5">
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Print Receipt</Dialog.Title>
				<Dialog.Description
					>Preview the receipt before sending it to the printer.</Dialog.Description
				>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				{#if isLoading}
					<div class="grid gap-3">
						<Skeleton class="mx-auto h-5 w-36" />
						<Skeleton class="h-5 w-full" />
						<Skeleton class="h-32 w-full" />
						<Skeleton class="h-5 w-full" />
					</div>
				{:else if errorMessage}
					<ErrorAlert title="Receipt failed." message={errorMessage} />
				{:else if sale}
					<div
						class="mx-auto w-full max-w-80 rounded-md border bg-white p-4 font-mono text-[11px] leading-relaxed text-slate-950 shadow-sm"
					>
						<div class="text-center">
							<div class="text-sm font-bold">POS SYSTEM</div>
							<div class="text-slate-500">Receipt</div>
						</div>

						<div class="my-3 border-t border-dashed border-slate-400"></div>

						<div class="grid gap-1">
							<div class="flex justify-between gap-3">
								<span>No</span>
								<span class="text-right">{sale.transaction_no}</span>
							</div>
							<div class="flex justify-between gap-3">
								<span>Date</span>
								<span class="text-right">{formatDateTime(sale.sold_at)}</span>
							</div>
							<div class="flex justify-between gap-3">
								<span>Cashier</span>
								<span class="text-right">{sale.cashier_name}</span>
							</div>
							<div class="flex justify-between gap-3">
								<span>Register</span>
								<span class="text-right">{sale.cash_register_code ?? '-'}</span>
							</div>
						</div>

						<div class="my-3 border-t border-dashed border-slate-400"></div>

						<Table.Root>
							<Table.Body>
								{#each sale.items as item (item.id)}
									<Table.Row class="border-0">
										<Table.Cell class="px-0 py-1">
											<div class="font-bold">{item.product_name}</div>
											<div class="text-slate-500">{item.product_sku} / {item.unit}</div>
											<div class="flex justify-between gap-3">
												<span>
													{formatNumber(item.quantity)} x {formatIdr(item.unit_price)}
												</span>
												<span>{formatIdr(item.line_total)}</span>
											</div>
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>

						<div class="my-3 border-t border-dashed border-slate-400"></div>

						<div class="grid gap-1">
							<div class="flex justify-between gap-3">
								<span>Subtotal</span>
								<span>{formatIdr(sale.subtotal_amount)}</span>
							</div>
							<div class="flex justify-between gap-3 text-sm font-bold">
								<span>Total</span>
								<span>{formatIdr(sale.total_amount)}</span>
							</div>
							<div class="flex justify-between gap-3">
								<span>Paid</span>
								<span>{formatIdr(sale.paid_amount)}</span>
							</div>
							<div class="flex justify-between gap-3">
								<span>Change</span>
								<span>{formatIdr(sale.change_amount)}</span>
							</div>
						</div>

						{#if sale.notes}
							<div class="my-3 border-t border-dashed border-slate-400"></div>
							<div>
								<div>Notes</div>
								<div>{sale.notes}</div>
							</div>
						{/if}

						<div class="my-3 border-t border-dashed border-slate-400"></div>
						<div class="text-center">
							<div>Terima kasih</div>
							<div class="text-slate-500">Barang yang sudah dibeli mengikuti kebijakan toko.</div>
						</div>
					</div>
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close type="button" class={cn(buttonVariants({ variant: 'outline' }))}>
					Close
				</Dialog.Close>
				<Button type="button" disabled={!sale || isLoading} onclick={handlePrint}>
					<PrinterIcon />
					Print receipt
				</Button>
			</Dialog.Footer>
		</div>
	</Dialog.Content>
</Dialog.Root>
