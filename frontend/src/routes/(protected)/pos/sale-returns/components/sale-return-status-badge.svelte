<script lang="ts">
	import type { SaleReturnStatus } from '$lib/api/sales.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: SaleReturnStatus } = $props();

	const labels = $derived<Record<SaleReturnStatus, string>>({ completed: t('pos.saleReturns.statusCompleted'),
		cancelled: t('pos.saleReturns.statusCancelled') });

	const classes: Record<SaleReturnStatus, string> = {
		completed:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cancelled: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[status]}>{labels[status]}</Badge>
