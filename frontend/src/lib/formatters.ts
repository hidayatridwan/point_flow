const dateTimeFormatter = new Intl.DateTimeFormat('en-US', {
	dateStyle: 'medium',
	timeStyle: 'short'
});

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
