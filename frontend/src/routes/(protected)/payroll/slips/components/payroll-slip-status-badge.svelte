<script lang="ts">
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { status }: { status: string } = $props();

	const labels = $derived<Record<string, string>>({
		completed: t('payroll.slips.statusCompleted'),
		cancelled: t('payroll.slips.statusCancelled')
	});

	const tones: Record<string, BadgeVariant> = {
		completed: 'default',
		cancelled: 'destructive'
	};
</script>

<Badge variant={tones[status] ?? 'ghost'}>{labels[status] ?? status}</Badge>
