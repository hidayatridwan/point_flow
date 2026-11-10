import { apiGet } from './client.js';

type DashboardResponse = {
	transaction_count_today: number;
	revenue_today: number;
	cash_revenue_today: number;
	noncash_revenue_today: number;
	low_stock_product_count: number;
	pending_purchase_order_count: number;
	draft_stock_adjustment_count: number;
	active_employee_count: number;
};

export type DashboardSummary = {
	transactionCountToday: number;
	revenueToday: number;
	cashRevenueToday: number;
	noncashRevenueToday: number;
	lowStockProductCount: number;
	pendingPurchaseOrderCount: number;
	draftStockAdjustmentCount: number;
	activeEmployeeCount: number;
};

export async function getDashboardSummary(): Promise<DashboardSummary> {
	const summary = await apiGet<DashboardResponse>('/dashboard');

	return {
		transactionCountToday: summary.transaction_count_today,
		revenueToday: summary.revenue_today,
		cashRevenueToday: summary.cash_revenue_today,
		noncashRevenueToday: summary.noncash_revenue_today,
		lowStockProductCount: summary.low_stock_product_count,
		pendingPurchaseOrderCount: summary.pending_purchase_order_count,
		draftStockAdjustmentCount: summary.draft_stock_adjustment_count,
		activeEmployeeCount: summary.active_employee_count
	};
}

// --- Sales trend (line / area chart) ---

type SalesTrendResponse = {
	date: string;
	revenue: number;
	transaction_count: number;
};

export type SalesTrendPoint = {
	date: string;
	revenue: number;
	transactionCount: number;
};

export async function getSalesTrend(days = 30): Promise<SalesTrendPoint[]> {
	const rows = await apiGet<SalesTrendResponse[]>('/dashboard/sales-trend', {
		params: { days }
	});

	return rows.map((row) => ({
		date: row.date,
		revenue: row.revenue,
		transactionCount: row.transaction_count
	}));
}

// --- Top products (horizontal bar chart) ---

type TopProductResponse = {
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	revenue: number;
	quantity_sold: number;
};

export type TopProductItem = {
	productSkuId: number;
	productName: string;
	productSku: string;
	unit: string;
	revenue: number;
	quantitySold: number;
};

export async function getTopProducts(days = 30, limit = 10): Promise<TopProductItem[]> {
	const rows = await apiGet<TopProductResponse[]>('/dashboard/top-products', {
		params: { days, limit }
	});

	return rows.map((row) => ({
		productSkuId: row.product_sku_id,
		productName: row.product_name,
		productSku: row.product_sku,
		unit: row.unit,
		revenue: row.revenue,
		quantitySold: row.quantity_sold
	}));
}

// --- Hourly sales (bar chart) ---

type HourlySalesResponse = {
	hour: number;
	revenue: number;
	transaction_count: number;
};

export type HourlySalesPoint = {
	hour: number;
	revenue: number;
	transactionCount: number;
};

export async function getHourlySales(date?: string): Promise<HourlySalesPoint[]> {
	const rows = await apiGet<HourlySalesResponse[]>('/dashboard/hourly-sales', {
		params: date ? { date } : {}
	});

	return rows.map((row) => ({
		hour: row.hour,
		revenue: row.revenue,
		transactionCount: row.transaction_count
	}));
}

// --- Low stock (data table) ---

type LowStockResponse = {
	product_sku_id: number;
	product_id: number;
	product_name: string;
	sku: string;
	barcode: string | null;
	category_name: string | null;
	purchase_price: number;
	selling_price: number;
	stock: number;
	minimum_stock: number;
	unit: string;
	quantity_on_order: number;
	is_low_stock: boolean;
	is_active: boolean;
};

export type LowStockItem = {
	productSkuId: number;
	productName: string;
	sku: string;
	categoryName: string | null;
	unit: string;
	stock: number;
	minimumStock: number;
	quantityOnOrder: number;
};

export async function getLowStock(limit = 20): Promise<LowStockItem[]> {
	const rows = await apiGet<LowStockResponse[]>('/dashboard/low-stock', {
		params: { limit }
	});

	return rows.map((row) => ({
		productSkuId: row.product_sku_id,
		productName: row.product_name,
		sku: row.sku,
		categoryName: row.category_name,
		unit: row.unit,
		stock: row.stock,
		minimumStock: row.minimum_stock,
		quantityOnOrder: row.quantity_on_order
	}));
}

// --- Sales by category (pie / donut chart) ---

type SalesByCategoryResponse = {
	category_name: string;
	revenue: number;
	transaction_count: number;
	percentage: number;
};

export type SalesByCategoryItem = {
	categoryName: string;
	revenue: number;
	transactionCount: number;
	percentage: number;
};

export async function getSalesByCategory(days = 30): Promise<SalesByCategoryItem[]> {
	const rows = await apiGet<SalesByCategoryResponse[]>('/dashboard/sales-by-category', {
		params: { days }
	});

	return rows.map((row) => ({
		categoryName: row.category_name,
		revenue: row.revenue,
		transactionCount: row.transaction_count,
		percentage: row.percentage
	}));
}

// --- Product scatter (scatter / bubble chart) ---

type ProductScatterResponse = {
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	selling_price: number;
	quantity_sold: number;
	revenue: number;
};

export type ProductScatterItem = {
	productSkuId: number;
	productName: string;
	productSku: string;
	unit: string;
	sellingPrice: number;
	quantitySold: number;
	revenue: number;
};

export async function getProductScatter(days = 30, limit = 50): Promise<ProductScatterItem[]> {
	const rows = await apiGet<ProductScatterResponse[]>('/dashboard/product-scatter', {
		params: { days, limit }
	});

	return rows.map((row) => ({
		productSkuId: row.product_sku_id,
		productName: row.product_name,
		productSku: row.product_sku,
		unit: row.unit,
		sellingPrice: row.selling_price,
		quantitySold: row.quantity_sold,
		revenue: row.revenue
	}));
}

// --- Cashier performance (bar chart) ---

type CashierPerformanceResponse = {
	cashier_id: number;
	cashier_name: string;
	transaction_count: number;
	revenue: number;
	avg_basket_size: number;
	return_count: number;
};

export type CashierPerformanceItem = {
	cashierId: number;
	cashierName: string;
	transactionCount: number;
	revenue: number;
	avgBasketSize: number;
	returnCount: number;
};

export async function getCashierPerformance(days = 30): Promise<CashierPerformanceItem[]> {
	const rows = await apiGet<CashierPerformanceResponse[]>('/dashboard/cashier-performance', {
		params: { days }
	});

	return rows.map((row) => ({
		cashierId: row.cashier_id,
		cashierName: row.cashier_name,
		transactionCount: row.transaction_count,
		revenue: row.revenue,
		avgBasketSize: row.avg_basket_size,
		returnCount: row.return_count
	}));
}

// --- Procurement funnel (funnel chart) ---

type ProcurementFunnelResponse = {
	po_draft: number;
	po_ordered: number;
	po_partially_received: number;
	po_received: number;
	po_cancelled: number;
	gr_draft: number;
	gr_posted: number;
	gr_cancelled: number;
	sa_draft: number;
	sa_posted: number;
	sa_cancelled: number;
};

export type ProcurementFunnel = {
	poDraft: number;
	poOrdered: number;
	poPartiallyReceived: number;
	poReceived: number;
	poCancelled: number;
	grDraft: number;
	grPosted: number;
	grCancelled: number;
	saDraft: number;
	saPosted: number;
	saCancelled: number;
};

export async function getProcurementFunnel(): Promise<ProcurementFunnel> {
	const data = await apiGet<ProcurementFunnelResponse>('/dashboard/procurement-funnel');

	return {
		poDraft: data.po_draft,
		poOrdered: data.po_ordered,
		poPartiallyReceived: data.po_partially_received,
		poReceived: data.po_received,
		poCancelled: data.po_cancelled,
		grDraft: data.gr_draft,
		grPosted: data.gr_posted,
		grCancelled: data.gr_cancelled,
		saDraft: data.sa_draft,
		saPosted: data.sa_posted,
		saCancelled: data.sa_cancelled
	};
}
