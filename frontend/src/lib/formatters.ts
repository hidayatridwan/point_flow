const dateTimeFormatter = new Intl.DateTimeFormat('en-US', {
	dateStyle: 'medium',
	timeStyle: 'short'
});

const dateFormatter = new Intl.DateTimeFormat('en-US', {
	dateStyle: 'medium'
});

export function formatDate(value: string | null | undefined) {
	if (!value) return '-';

	const date = new Date(`${value}T00:00:00`);
	if (Number.isNaN(date.getTime())) return '-';
	return dateFormatter.format(date);
}

export function formatDateTime(value: string | null | undefined) {
	if (!value) return '-';

	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return '-';
	return dateTimeFormatter.format(date);
}

const idrFormatter = new Intl.NumberFormat('en-US', {
	style: 'currency',
	currency: 'IDR',
	maximumFractionDigits: 0
});

export function formatIdr(value: number | null | undefined) {
	if (value === null || value === undefined || Number.isNaN(value)) return '-';
	return idrFormatter.format(value);
}

const numberFormatter = new Intl.NumberFormat('en-US', {
	maximumFractionDigits: 3
});

export function formatNumber(value: number | null | undefined) {
	if (value === null || value === undefined || Number.isNaN(value)) return '-';
	return numberFormatter.format(value);
}
