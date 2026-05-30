<script lang="ts">
	import type { CashDrawerMovementStatus } from '$lib/api/cash-registers.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: CashDrawerMovementStatus } = $props();

	const labels = $derived<Record<CashDrawerMovementStatus, string>>({ completed: t('pos.cashMovements.statusCompleted'),
		cancelled: t('pos.cashMovements.statusCancelled') });

	const classes: Record<CashDrawerMovementStatus, string> = {
		completed:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cancelled: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[status]}>{labels[status]}</Badge>
