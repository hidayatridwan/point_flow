// Shared helpers for dashboard chart components.

// Palette wired to the `--chart-*` CSS variables defined in routes/layout.css.
// Used to color discrete categories (pie slices, funnel stages, etc.).
// Assign in order, never cycled: extra categories fold into "Other" instead.
export const CHART_COLORS = [
	'var(--chart-1)',
	'var(--chart-2)',
	'var(--chart-3)',
	'var(--chart-4)',
	'var(--chart-5)'
];

// Recessive neutral for the folded "Other" slice and cancelled stages.
export const CHART_OTHER_COLOR = 'var(--chart-other)';

// Vertical bar fill: indigo at the top fading to purple at the baseline.
export const BAR_GRADIENT_STOPS = ['var(--chart-gradient-from)', 'var(--chart-gradient-to)'];

// Same gradient for horizontal HTML bars (progress-style rows).
export const BAR_GRADIENT_CSS =
	'linear-gradient(90deg, var(--chart-gradient-from), var(--chart-gradient-to))';

// One-hue ordinal ramp for ordered stages (draft → completed), light → dark.
export const STAGE_RAMP = [
	'var(--chart-ramp-1)',
	'var(--chart-ramp-2)',
	'var(--chart-ramp-3)',
	'var(--chart-ramp-4)'
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

const percentFormatter = new Intl.NumberFormat('en-US', {
	style: 'percent',
	maximumFractionDigits: 1
});

// Share on a 0-100 scale (as returned by the API), e.g. 42.37 -> "42.4%".
export function formatPercent(value: number | null | undefined): string {
	if (value === null || value === undefined || Number.isNaN(value)) return '0%';
	return percentFormatter.format(value / 100);
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
