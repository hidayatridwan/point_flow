<script lang="ts">
	import { getAuditLog, type AuditLog } from '$lib/api/audit-logs.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import AuditJsonBlock from './audit-json-block.svelte';
	import AuditLogOutcomeBadge from './audit-log-outcome-badge.svelte';

	let {
		auditLogId,
		open = $bindable(false)
	}: {
		auditLogId: number;
		open: boolean;
	} = $props();

	let auditLog = $state<AuditLog | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadAuditLog(auditLogId);
	});

	async function loadAuditLog(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getAuditLog(id);
			if (requestId !== latestRequestId) return;
			auditLog = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			auditLog = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getActorLabel(log: AuditLog) {
		if (log.actor_name && log.actor_email) return `${log.actor_name} <${log.actor_email}>`;
		if (log.actor_name) return log.actor_name;
		if (log.actor_email) return log.actor_email;
		return log.actor_id ? `Actor #${log.actor_id}` : 'System';
	}

	function getResourceLabel(log: AuditLog) {
		const id = log.resource_id ? ` #${log.resource_id}` : '';
		return `${log.resource_type}${id}`;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load audit log details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-5xl">
		<Dialog.Header>
			<Dialog.Title>Audit Log Details</Dialog.Title>
			<Dialog.Description>Immutable event record captured by the backend.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-40 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if auditLog}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="Audit Log ID" value={auditLog.id} />
					<DetailField label="Outcome">
						<AuditLogOutcomeBadge outcome={auditLog.outcome} />
					</DetailField>
					<DetailField label="Occurred At" value={formatDateTime(auditLog.occurred_at)} />
					<DetailField label="Action" value={auditLog.action} />
					<DetailField label="Resource" value={getResourceLabel(auditLog)} />
					<DetailField label="Resource Label" value={auditLog.resource_label || '-'} />
					<DetailField label="Actor" value={getActorLabel(auditLog)} />
					<DetailField label="Actor Role" value={auditLog.actor_role || '-'} />
					<DetailField label="Request ID" value={auditLog.request_id || '-'} />
					<DetailField label="IP Address" value={auditLog.ip_address || '-'} />
					<DetailField label="User Agent" value={auditLog.user_agent || '-'} />
				</div>

				<div class="grid gap-4 lg:grid-cols-3">
					<AuditJsonBlock label="Before Data" value={auditLog.before_data} />
					<AuditJsonBlock label="After Data" value={auditLog.after_data} />
					<AuditJsonBlock label="Metadata" value={auditLog.metadata} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
