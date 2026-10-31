<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Supplier } from '$lib/api/suppliers.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import SupplierDeleteAlertDialog from './supplier-delete-alert-dialog.svelte';
	import SupplierDetailDialog from './supplier-detail-dialog.svelte';
	import SupplierEditDialog from './supplier-edit-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		supplier,
		onDeleted,
		onUpdated
	}: {
		supplier: Supplier;
		onDeleted?: (supplierId: number) => void | Promise<void>;
		onUpdated?: (supplier: Supplier) => void;
	} = $props();

	let isDetailOpen = $state(false);
	let isEditOpen = $state(false);
	let isDeleteOpen = $state(false);
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
					aria-label={`${t('common.action')}: ${supplier.name}`}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={() => (isDetailOpen = true)}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={() => (isEditOpen = true)}>
				<PencilIcon />
				{t('common.edit')}
			</DropdownMenu.Item>
			<DropdownMenu.Item variant="destructive" onSelect={() => (isDeleteOpen = true)}>
				<TrashIcon />
				{t('common.delete')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<SupplierDetailDialog supplierId={supplier.id} bind:open={isDetailOpen} />
	<SupplierEditDialog {supplier} bind:open={isEditOpen} {onUpdated} />
	<SupplierDeleteAlertDialog {supplier} bind:open={isDeleteOpen} {onDeleted} />
</div>
