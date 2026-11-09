<script lang="ts">
	import type { FinancialTransactionStatus } from '$lib/api/finances.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: FinancialTransactionStatus } = $props();

	const labels = $derived<Record<FinancialTransactionStatus, string>>({
		completed: t('finances.transactions.statusCompleted'),
		cancelled: t('finances.transactions.statusCancelled')
	});

	const tones: Record<FinancialTransactionStatus, BadgeVariant> = {
		completed: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
