<script lang="ts">
	import type { CashDrawerMovementType } from '$lib/api/cash-registers.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { type }: { type: CashDrawerMovementType } = $props();

	const labels = $derived<Record<CashDrawerMovementType, string>>({ cash_in: t('pos.cashMovements.typeCashIn'),
		cash_drop: t('pos.cashMovements.typeCashDrop'),
		cash_out: t('pos.cashMovements.typeCashOut'),
		petty_cash: t('pos.cashMovements.typePettyCash') });

	const classes: Record<CashDrawerMovementType, string> = {
		cash_in:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		cash_out:
			'border-amber-600/20 bg-amber-50 text-amber-700 dark:bg-amber-950/30 dark:text-amber-300',
		cash_drop:
			'border-sky-600/20 bg-sky-50 text-sky-700 dark:bg-sky-950/30 dark:text-sky-300',
		petty_cash:
			'border-violet-600/20 bg-violet-50 text-violet-700 dark:bg-violet-950/30 dark:text-violet-300'
	};
</script>

<Badge variant="outline" class={classes[type]}>{labels[type]}</Badge>
