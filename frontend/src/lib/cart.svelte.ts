import { browser } from '$app/environment';

export type CartLine = {
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	unit_price: number;
	photo_url: string | null;
	stock: number;
	quantity: number;
};

export type SavedCartLine = {
	product_sku_id: number;
	quantity: number;
};

const STORAGE_KEY = 'pointflow:cart:v1';

function isValidLine(value: unknown): value is CartLine {
	if (!value || typeof value !== 'object') return false;
	const line = value as Record<string, unknown>;
	return (
		typeof line.product_sku_id === 'number' &&
		Number.isFinite(line.product_sku_id) &&
		typeof line.product_name === 'string' &&
		typeof line.product_sku === 'string' &&
		typeof line.unit === 'string' &&
		typeof line.unit_price === 'number' &&
		Number.isFinite(line.unit_price) &&
		(line.photo_url === null || typeof line.photo_url === 'string') &&
		typeof line.stock === 'number' &&
		typeof line.quantity === 'number' &&
		Number.isFinite(line.quantity) &&
		line.quantity > 0
	);
}

function loadCart(): CartLine[] {
	if (!browser) return [];
	try {
		const raw = window.localStorage.getItem(STORAGE_KEY);
		if (!raw) return [];
		const parsed: unknown = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];
		return parsed.filter(isValidLine);
	} catch {
		return [];
	}
}

function persistCart(lines: CartLine[]) {
	if (!browser) return;
	try {
		window.localStorage.setItem(STORAGE_KEY, JSON.stringify(lines));
	} catch {
		// Storage penuh atau diblokir: cart tetap hidup di memori sesi ini.
	}
}

export const cartState = $state<{ lines: CartLine[] }>({
	lines: loadCart()
});

export function reloadCartFromStorage() {
	cartState.lines = loadCart();
}

export function addToCart(line: Omit<CartLine, 'quantity'>, quantity = 1) {
	const existing = cartState.lines.find((item) => item.product_sku_id === line.product_sku_id);
	if (existing) {
		existing.quantity = Math.min(existing.quantity + quantity, Math.max(line.stock, 1));
		existing.unit_price = line.unit_price;
		existing.product_name = line.product_name;
		existing.photo_url = line.photo_url;
		existing.stock = line.stock;
	} else {
		cartState.lines.push({
			...line,
			quantity: Math.min(Math.max(quantity, 1), Math.max(line.stock, 1))
		});
	}
	persistCart(cartState.lines);
}

export function setLineQuantity(product_sku_id: number, quantity: number) {
	if (quantity <= 0) {
		removeLine(product_sku_id);
		return;
	}
	const existing = cartState.lines.find((item) => item.product_sku_id === product_sku_id);
	if (!existing) return;
	existing.quantity = quantity;
	persistCart(cartState.lines);
}

export function removeLine(product_sku_id: number) {
	cartState.lines = cartState.lines.filter((item) => item.product_sku_id !== product_sku_id);
	persistCart(cartState.lines);
}

export function clearCart() {
	cartState.lines = [];
	persistCart(cartState.lines);
}

export function cartCount(): number {
	return cartState.lines.reduce((total, line) => total + line.quantity, 0);
}

export function cartEstimate(): number {
	return cartState.lines.reduce((total, line) => total + line.quantity * line.unit_price, 0);
}
