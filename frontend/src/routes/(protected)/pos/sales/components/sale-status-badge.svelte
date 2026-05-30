<script lang="ts">
	import type { SaleStatus } from '$lib/api/sales.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { status }: { status: SaleStatus } = $props();

	const labels = $derived<Record<SaleStatus, string>>({ completed: t('pos.sales.statusCompleted'),
		cancelled: t('pos.sales.statusCancelled') });

	const classes: Record<SaleStatus, string> = {
		completed:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cancelled: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[status]}>{labels[status]}</Badge>
