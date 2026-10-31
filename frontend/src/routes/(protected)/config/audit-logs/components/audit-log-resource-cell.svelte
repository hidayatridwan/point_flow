<script lang="ts">
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		resourceType,
		resourceId,
		resourceLabel
	}: {
		resourceType: string;
		resourceId: number | null;
		resourceLabel: string | null;
	} = $props();

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
</script>

<div class="grid min-w-36 gap-0.5">
	<div class="truncate font-medium">{getResourceTypeLabel(resourceType)}</div>
	<div class="truncate text-xs text-muted-foreground">
		{resourceLabel || (resourceId ? `ID ${resourceId}` : '-')}
	</div>
</div>
