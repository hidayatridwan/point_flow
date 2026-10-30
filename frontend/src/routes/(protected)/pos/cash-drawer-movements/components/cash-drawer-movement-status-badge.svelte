<script lang="ts">
	import type { CashDrawerMovementStatus } from '$lib/api/cash-registers.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: CashDrawerMovementStatus } = $props();

	const labels = $derived<Record<CashDrawerMovementStatus, string>>({
		completed: t('pos.cashMovements.statusCompleted'),
		cancelled: t('pos.cashMovements.statusCancelled')
	});

	const tones: Record<CashDrawerMovementStatus, BadgeVariant> = {
		completed: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
