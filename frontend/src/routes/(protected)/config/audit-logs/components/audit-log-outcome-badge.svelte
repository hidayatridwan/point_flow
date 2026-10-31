<script lang="ts">
	import type { AuditLogOutcome } from '$lib/api/audit-logs.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { outcome }: { outcome: AuditLogOutcome } = $props();

	const labels = $derived<Record<AuditLogOutcome, string>>({
		success: t('config.auditLogs.outcomeSuccess'),
		failure: t('config.auditLogs.outcomeFailure')
	});

	const tones: Record<AuditLogOutcome, BadgeVariant> = {
		success: 'default',
		failure: 'destructive'
	};
</script>

<Badge variant={tones[outcome]}>{labels[outcome]}</Badge>
