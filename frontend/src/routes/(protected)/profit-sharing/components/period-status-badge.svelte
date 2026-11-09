<script lang="ts">
	import type { ProfitSharingPeriodStatus } from '$lib/api/profit-sharing.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let { status }: { status: ProfitSharingPeriodStatus } = $props();

	const labels = $derived<Record<ProfitSharingPeriodStatus, string>>({
		draft: t('finances.profitSharing.statusDraft'),
		calculated: t('finances.profitSharing.statusCalculated'),
		finalized: t('finances.profitSharing.statusFinalized'),
		cancelled: t('finances.profitSharing.statusCancelled')
	});

	const tones: Record<ProfitSharingPeriodStatus, BadgeVariant> = {
		draft: 'secondary',
		calculated: 'outline',
		finalized: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status]}>{labels[status]}</Badge>
