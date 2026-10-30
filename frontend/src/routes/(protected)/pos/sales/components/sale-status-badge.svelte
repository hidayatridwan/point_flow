<script lang="ts">
	import type { SaleStatus } from '$lib/api/sales.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { status }: { status: SaleStatus } = $props();

	const labels = $derived<Record<SaleStatus, string>>({
		completed: t('pos.sales.statusCompleted'),
		cancelled: t('pos.sales.statusCancelled')
	});

	const tones: Record<SaleStatus, BadgeVariant> = {
		completed: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
