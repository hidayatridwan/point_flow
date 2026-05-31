<script lang="ts">
	import type { CashDrawerMovementType } from '$lib/api/cash-registers.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { type }: { type: CashDrawerMovementType } = $props();

	const labels = $derived<Record<CashDrawerMovementType, string>>({
		cash_in: t('pos.cashMovements.typeCashIn'),
		cash_drop: t('pos.cashMovements.typeCashDrop'),
		cash_out: t('pos.cashMovements.typeCashOut'),
		petty_cash: t('pos.cashMovements.typePettyCash')
	});

	const tones: Record<CashDrawerMovementType, BadgeVariant> = {
		cash_in: 'success',
		cash_out: 'warning',
		cash_drop: 'info',
		petty_cash: 'accent'
	};
</script>

<Badge variant={tones[type]}>{labels[type]}</Badge>
