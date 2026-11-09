<script lang="ts">
	import type { FinancialTransactionType } from '$lib/api/finances.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { type }: { type: FinancialTransactionType } = $props();

	const labels = $derived<Record<FinancialTransactionType, string>>({
		income: t('finances.transactions.typeIncome'),
		expense: t('finances.transactions.typeExpense')
	});

	const tones: Record<FinancialTransactionType, BadgeVariant> = {
		income: 'default',
		expense: 'secondary'
	};
</script>

<Badge variant={tones[type]}>{labels[type]}</Badge>
