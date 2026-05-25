<script lang="ts">
	import type { AuditLogOutcome } from '$lib/api/audit-logs.js';
	import { Badge } from '$lib/components/ui/badge/index.js';

	let { outcome }: { outcome: AuditLogOutcome } = $props();

	const labels: Record<AuditLogOutcome, string> = {
		success: 'Success',
		failure: 'Failure'
	};

	const classes: Record<AuditLogOutcome, string> = {
		success:
			'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300',
		failure: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'
	};
</script>

<Badge variant="outline" class={classes[outcome]}>{labels[outcome]}</Badge>
