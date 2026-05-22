import { apiGet } from './client.js';

type DashboardResponse = {
	transaction_count_today: number;
	revenue_today: number;
	low_stock_product_count: number;
	pending_purchase_order_count: number;
	draft_stock_adjustment_count: number;
	active_employee_count: number;
};

export type DashboardSummary = {
	transactionCountToday: number;
	revenueToday: number;
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
		lowStockProductCount: summary.low_stock_product_count,
		pendingPurchaseOrderCount: summary.pending_purchase_order_count,
		draftStockAdjustmentCount: summary.draft_stock_adjustment_count,
		activeEmployeeCount: summary.active_employee_count
	};
}
