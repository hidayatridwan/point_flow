import type { SaleDetail, SaleItem } from '$lib/api/sales.js';

const idrFormatter = new Intl.NumberFormat('id-ID', {
	style: 'currency',
	currency: 'IDR',
	maximumFractionDigits: 0
});

const numberFormatter = new Intl.NumberFormat('id-ID', {
	maximumFractionDigits: 3
});

const dateTimeFormatter = new Intl.DateTimeFormat('id-ID', {
	dateStyle: 'medium',
	timeStyle: 'short'
});

export function printSaleReceipt(sale: SaleDetail) {
	if (typeof window === 'undefined') return;

	const printWindow = window.open('', '_blank', 'popup,width=420,height=720');
	if (!printWindow) {
		throw new Error('Unable to open print window. Allow pop-ups and try again.');
	}

	printWindow.document.open();
	printWindow.document.write(buildSaleReceiptHtml(sale));
	printWindow.document.close();
	printWindow.focus();
	printWindow.setTimeout(() => {
		printWindow.print();
	}, 250);
}

function buildSaleReceiptHtml(sale: SaleDetail) {
	const itemRows = sale.items.map(buildItemRow).join('');

	return `<!doctype html>
<html lang="id">
<head>
	<meta charset="utf-8" />
	<title>${escapeHtml(sale.transaction_no)} Receipt</title>
	<style>
		@page {
			size: 80mm auto;
			margin: 0;
		}

		* {
			box-sizing: border-box;
		}

		body {
			margin: 0;
			background: #fff;
			color: #111;
			font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
			font-size: 11px;
			line-height: 1.35;
		}

		.receipt {
			width: 80mm;
			padding: 5mm 4mm;
		}

		.center {
			text-align: center;
		}

		.store-name {
			font-size: 15px;
			font-weight: 700;
			letter-spacing: 0;
		}

		.muted {
			color: #444;
		}

		.divider {
			border: 0;
			border-top: 1px dashed #111;
			margin: 8px 0;
		}

		.row {
			display: flex;
			justify-content: space-between;
			gap: 8px;
		}

		.row > :first-child {
			min-width: 0;
		}

		.value {
			text-align: right;
			white-space: nowrap;
		}

		.item {
			display: grid;
			gap: 2px;
			margin-bottom: 7px;
		}

		.item-name {
			font-weight: 700;
			overflow-wrap: anywhere;
		}

		.item-meta {
			display: flex;
			justify-content: space-between;
			gap: 8px;
		}

		.total {
			font-size: 13px;
			font-weight: 700;
		}

		.footer {
			margin-top: 10px;
			text-align: center;
		}
	</style>
</head>
<body>
	<main class="receipt">
		<section class="center">
			<div class="store-name">POS SYSTEM</div>
			<div class="muted">Receipt</div>
		</section>

		<hr class="divider" />

		<section>
			<div class="row"><span>No</span><span class="value">${escapeHtml(sale.transaction_no)}</span></div>
			<div class="row"><span>Date</span><span class="value">${escapeHtml(formatDateTime(sale.sold_at))}</span></div>
			<div class="row"><span>Cashier</span><span class="value">${escapeHtml(sale.cashier_name)}</span></div>
			<div class="row"><span>Register</span><span class="value">${escapeHtml(sale.cash_register_code ?? '-')}</span></div>
			<div class="row"><span>Status</span><span class="value">${escapeHtml(formatStatus(sale.status))}</span></div>
		</section>

		<hr class="divider" />

		<section>
			${itemRows}
		</section>

		<hr class="divider" />

		<section>
			<div class="row"><span>Subtotal</span><span class="value">${escapeHtml(formatIdr(sale.subtotal_amount))}</span></div>
			<div class="row total"><span>Total</span><span class="value">${escapeHtml(formatIdr(sale.total_amount))}</span></div>
			<div class="row"><span>Paid</span><span class="value">${escapeHtml(formatIdr(sale.paid_amount))}</span></div>
			<div class="row"><span>Change</span><span class="value">${escapeHtml(formatIdr(sale.change_amount))}</span></div>
		</section>

		${sale.notes ? `<hr class="divider" /><section><div>Notes</div><div>${escapeHtml(sale.notes)}</div></section>` : ''}

		<hr class="divider" />

		<section class="footer">
			<div>Terima kasih</div>
			<div class="muted">Barang yang sudah dibeli mengikuti kebijakan toko.</div>
		</section>
	</main>
</body>
</html>`;
}

function buildItemRow(item: SaleItem) {
	return `<div class="item">
	<div class="item-name">${escapeHtml(item.product_name)}</div>
	<div class="muted">${escapeHtml(item.product_sku)} / ${escapeHtml(item.unit)}</div>
	<div class="item-meta">
		<span>${escapeHtml(formatNumber(item.quantity))} x ${escapeHtml(formatIdr(item.unit_price))}</span>
		<span class="value">${escapeHtml(formatIdr(item.line_total))}</span>
	</div>
</div>`;
}

function formatDateTime(value: string | null | undefined) {
	if (!value) return '-';
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return '-';
	return dateTimeFormatter.format(date);
}

function formatIdr(value: number | null | undefined) {
	if (value === null || value === undefined || Number.isNaN(value)) return '-';
	return idrFormatter.format(value);
}

function formatNumber(value: number | null | undefined) {
	if (value === null || value === undefined || Number.isNaN(value)) return '-';
	return numberFormatter.format(value);
}

function formatStatus(status: string) {
	return status.charAt(0).toUpperCase() + status.slice(1);
}

function escapeHtml(value: string) {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;');
}
