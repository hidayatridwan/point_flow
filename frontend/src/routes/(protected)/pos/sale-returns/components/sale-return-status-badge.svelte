<script lang="ts">
	import type { SaleReturnStatus } from '$lib/api/sales.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: SaleReturnStatus } = $props();

	const labels = $derived<Record<SaleReturnStatus, string>>({
		completed: t('pos.saleReturns.statusCompleted'),
		cancelled: t('pos.saleReturns.statusCancelled')
	});

	const tones: Record<SaleReturnStatus, BadgeVariant> = {
		completed: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
