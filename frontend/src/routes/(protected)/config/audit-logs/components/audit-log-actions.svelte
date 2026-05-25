<script lang="ts">
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { AuditLog } from '$lib/api/audit-logs.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import AuditLogDetailDialog from './audit-log-detail-dialog.svelte';

	let { auditLog }: { auditLog: AuditLog } = $props();

	let isDetailOpen = $state(false);

	function openDetail() {
		isDetailOpen = true;
	}
</script>

<div class="flex justify-center">
	<Button
		type="button"
		variant="ghost"
		size="icon-sm"
		class="size-8"
		aria-label={`Open audit log ${auditLog.id}`}
		title="Detail"
		onclick={openDetail}
	>
		<EyeIcon />
	</Button>

	<AuditLogDetailDialog auditLogId={auditLog.id} bind:open={isDetailOpen} />
</div>
