// Shared helpers for dashboard chart components.

// Palette wired to the `--chart-*` CSS variables defined in routes/layout.css.
// Used to color discrete categories (pie slices, funnel stages, etc.).
export const CHART_COLORS = [
	'var(--chart-1)',
	'var(--chart-2)',
	'var(--chart-3)',
	'var(--chart-4)',
	'var(--chart-5)'
];

const compactFormatter = new Intl.NumberFormat('en-US', {
	notation: 'compact',
	maximumFractionDigits: 1
});

// Compact axis labels, e.g. 1.2K / 3.4M. Keeps long IDR amounts readable on axes.
export function formatCompact(value: number | null | undefined): string {
	if (value === null || value === undefined || Number.isNaN(value)) return '0';
	return compactFormatter.format(value);
}

const shortDateFormatter = new Intl.DateTimeFormat('en-US', {
	day: 'numeric',
	month: 'short'
});

export function formatShortDate(value: Date | string): string {
	const date = typeof value === 'string' ? new Date(value) : value;
	if (Number.isNaN(date.getTime())) return '-';
	return shortDateFormatter.format(date);
}

// Hour-of-day label, e.g. 8 -> "08:00".
export function formatHour(hour: number): string {
	return `${String(hour).padStart(2, '0')}:00`;
}
