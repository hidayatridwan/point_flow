<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { PayrollSlip } from '$lib/api/payroll.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import PayrollSlipDetailDialog from './payroll-slip-detail-dialog.svelte';

	let {
		payrollSlip
	}: {
		payrollSlip: PayrollSlip;
	} = $props();

	let isDetailOpen = $state(false);

	function openDetail() {
		isDetailOpen = true;
	}
</script>

<div class="flex justify-center">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="ghost"
					size="icon-sm"
					class="size-8"
					aria-label={t('common.openActions').replace('{docNo}', payrollSlip.slip_no)}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-44">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				<span class="min-w-0 flex-1 whitespace-nowrap">{t('common.detail')}</span>
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<PayrollSlipDetailDialog payrollSlipId={payrollSlip.id} bind:open={isDetailOpen} />
</div>
