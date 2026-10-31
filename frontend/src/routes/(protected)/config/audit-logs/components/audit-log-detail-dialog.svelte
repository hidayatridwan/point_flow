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
	import { t } from '$lib/i18n/locale.svelte.js';

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
		return log.actor_id
			? t('config.auditLogs.actorLabel').replace('{id}', String(log.actor_id))
			: t('common.systemActor');
	}

	function getActionLabel(action: string) {
		switch (action) {
			case 'auth.login':
				return t('config.auditLogs.actionLogin');
			case 'auth.logout':
				return t('config.auditLogs.actionLogout');
			case 'auth.bootstrap_admin':
				return t('config.auditLogs.actionBootstrapAdmin');
			case 'auth.change_password':
				return t('config.auditLogs.actionChangePassword');
			case 'cash_register_session.open':
				return t('config.auditLogs.actionOpenRegister');
			case 'cash_register_session.close':
				return t('config.auditLogs.actionCloseRegister');
			case 'cash_drawer_movement.create':
				return t('config.auditLogs.actionCreateCashMovement');
			case 'cash_drawer_movement.cancel':
				return t('config.auditLogs.actionCancelCashMovement');
			case 'sale.create':
				return t('config.auditLogs.actionCreateSale');
			case 'sale.cancel':
				return t('config.auditLogs.actionCancelSale');
			case 'sale_return.create':
				return t('config.auditLogs.actionCreateReturn');
			case 'sale_return.cancel':
				return t('config.auditLogs.actionCancelReturn');
			default:
				return action;
		}
	}

	function getResourceTypeLabel(type: string) {
		switch (type) {
			case 'user':
				return t('config.auditLogs.resourceUser');
			case 'cash_register_session':
				return t('config.auditLogs.resourceRegisterSession');
			case 'cash_drawer_movement':
				return t('config.auditLogs.resourceCashMovement');
			case 'sale':
				return t('config.auditLogs.resourceSale');
			case 'sale_return':
				return t('config.auditLogs.resourceSaleReturn');
			default:
				return type;
		}
	}

	function getResourceLabel(log: AuditLog) {
		const id = log.resource_id ? ` #${log.resource_id}` : '';
		return `${getResourceTypeLabel(log.resource_type)}${id}`;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('config.auditLogs.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-5xl">
		<Dialog.Header>
			<Dialog.Title>{t('config.auditLogs.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('config.auditLogs.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-40 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if auditLog}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('config.auditLogs.logId')} value={auditLog.id} />
					<DetailField label={t('config.auditLogs.filterOutcome')}>
						<AuditLogOutcomeBadge outcome={auditLog.outcome} />
					</DetailField>
					<DetailField
						label={t('common.occurredAt')}
						value={formatDateTime(auditLog.occurred_at)}
					/>
					<DetailField
						label={t('config.auditLogs.filterAction')}
						value={getActionLabel(auditLog.action)}
					/>
					<DetailField
						label={t('config.auditLogs.filterResource')}
						value={getResourceLabel(auditLog)}
					/>
					<DetailField
						label={t('config.auditLogs.resourceLabel')}
						value={auditLog.resource_label || '-'}
					/>
					<DetailField label={t('config.auditLogs.actor')} value={getActorLabel(auditLog)} />
					<DetailField label={t('config.auditLogs.actorRole')} value={auditLog.actor_role || '-'} />
					<DetailField
						label={t('config.auditLogs.requestIdPlaceholder')}
						value={auditLog.request_id || '-'}
					/>
					<DetailField label={t('config.auditLogs.ipAddress')} value={auditLog.ip_address || '-'} />
					<DetailField label={t('config.auditLogs.userAgent')} value={auditLog.user_agent || '-'} />
				</div>

				<div class="grid gap-4 lg:grid-cols-3">
					<AuditJsonBlock label={t('config.auditLogs.beforeData')} value={auditLog.before_data} />
					<AuditJsonBlock label={t('config.auditLogs.afterData')} value={auditLog.after_data} />
					<AuditJsonBlock label={t('config.auditLogs.metadata')} value={auditLog.metadata} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
