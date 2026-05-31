<script lang="ts">
	import type { AuditLogOutcome } from '$lib/api/audit-logs.js';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge/index.js';

	let { outcome }: { outcome: AuditLogOutcome } = $props();

	const labels: Record<AuditLogOutcome, string> = {
		success: 'Success',
		failure: 'Failure'
	};

	const tones: Record<AuditLogOutcome, BadgeVariant> = {
		success: 'success',
		failure: 'destructive'
	};
</script>

<Badge variant={tones[outcome]}>{labels[outcome]}</Badge>
