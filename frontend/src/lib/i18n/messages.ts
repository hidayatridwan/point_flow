export const locales = ['id', 'en'] as const;

export type Locale = (typeof locales)[number];

export const localeLabels: Record<Locale, string> = {
	id: 'ID',
	en: 'EN'
};

export const messages = {
	id: {
		app: {
			name: 'POS System',
			subtitle: 'Kasir & Inventori'
		},
		language: {
			switchToEnglish: 'Ganti bahasa ke Inggris',
			switchToIndonesian: 'Ganti bahasa ke Indonesia'
		},
		theme: {
			switchToDark: 'Ganti ke mode gelap',
			switchToLight: 'Ganti ke mode terang'
		},
		common: {
			account: 'Akun',
			notifications: 'Notifikasi',
			logOut: 'Keluar',
			loggingOut: 'Keluar...',
			toggle: 'Toggle'
		},
		session: {
			openingSignIn: 'Membuka halaman masuk...',
			checkingSession: 'Memeriksa sesi...',
			pleaseWait: 'Mohon tunggu sebentar.'
		},
		navGroup: {
			overview: 'Ringkasan',
			core: 'Utama',
			config: 'Konfigurasi'
		},
		nav: {
			account: 'Akun',
			auditLogs: 'Audit Log',
			cashier: 'Kasir',
			cashMovements: 'Pergerakan Kas',
			cashRegisters: 'Register Kas',
			config: 'Konfigurasi',
			currentUser: 'User Saat Ini',
			dailySales: 'Penjualan Harian',
			dashboard: 'Dashboard',
			employees: 'Karyawan',
			goodsReceipts: 'Penerimaan Barang',
			helpDesk: 'Help Desk',
			inventory: 'Inventori',
			masterData: 'Master Data',
			monthlySales: 'Penjualan Bulanan',
			newTransaction: 'Transaksi Baru',
			payroll: 'Payroll',
			payrollReport: 'Laporan Payroll',
			payrollSlips: 'Slip Payroll',
			pointOfSale: 'Point of Sale',
			procurement: 'Pembelian',
			productCategories: 'Kategori Produk',
			products: 'Produk',
			productSkus: 'SKU Produk',
			purchaseOrders: 'Purchase Order',
			registerSession: 'Sesi Register',
			reports: 'Laporan',
			roles: 'Role',
			saleReturns: 'Retur Penjualan',
			salesHistory: 'Riwayat Penjualan',
			settings: 'Pengaturan',
			stockAdjustments: 'Penyesuaian Stok',
			stockConversions: 'Konversi Stok',
			stockMovements: 'Mutasi Stok',
			stockReport: 'Laporan Stok',
			summary: 'Ringkasan',
			suppliers: 'Supplier',
			units: 'Satuan',
			users: 'User'
		}
	},
	en: {
		app: {
			name: 'POS System',
			subtitle: 'Cashier & Inventory'
		},
		language: {
			switchToEnglish: 'Switch language to English',
			switchToIndonesian: 'Switch language to Indonesian'
		},
		theme: {
			switchToDark: 'Switch to dark mode',
			switchToLight: 'Switch to light mode'
		},
		common: {
			account: 'Account',
			notifications: 'Notifications',
			logOut: 'Log out',
			loggingOut: 'Logging out...',
			toggle: 'Toggle'
		},
		session: {
			openingSignIn: 'Opening sign in...',
			checkingSession: 'Checking session...',
			pleaseWait: 'Please wait a moment.'
		},
		navGroup: {
			overview: 'Overview',
			core: 'Core',
			config: 'Config'
		},
		nav: {
			account: 'Account',
			auditLogs: 'Audit Logs',
			cashier: 'Cashier',
			cashMovements: 'Cash Movements',
			cashRegisters: 'Cash Registers',
			config: 'Config',
			currentUser: 'Current User',
			dailySales: 'Daily Sales',
			dashboard: 'Dashboard',
			employees: 'Employees',
			goodsReceipts: 'Goods Receipts',
			helpDesk: 'Help Desk',
			inventory: 'Inventory',
			masterData: 'Master Data',
			monthlySales: 'Monthly Sales',
			newTransaction: 'New Transaction',
			payroll: 'Payroll',
			payrollReport: 'Payroll Report',
			payrollSlips: 'Payroll Slips',
			pointOfSale: 'Point of Sale',
			procurement: 'Procurement',
			productCategories: 'Product Categories',
			products: 'Products',
			productSkus: 'Product SKUs',
			purchaseOrders: 'Purchase Orders',
			registerSession: 'Register Session',
			reports: 'Reports',
			roles: 'Roles',
			saleReturns: 'Sale Returns',
			salesHistory: 'Sales History',
			settings: 'Settings',
			stockAdjustments: 'Stock Adjustments',
			stockConversions: 'Stock Conversions',
			stockMovements: 'Stock Movements',
			stockReport: 'Stock Report',
			summary: 'Summary',
			suppliers: 'Suppliers',
			units: 'Units',
			users: 'Users'
		}
	}
} as const;

export function isLocale(value: string | null | undefined): value is Locale {
	return locales.includes(value as Locale);
}
