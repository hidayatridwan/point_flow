<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Product } from '$lib/api/products.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import ProductDeleteAlertDialog from './product-delete-alert-dialog.svelte';
	import ProductDetailDialog from './product-detail-dialog.svelte';
	import ProductEditDialog from './product-edit-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		product,
		onDeleted,
		onUpdated
	}: {
		product: Product;
		onDeleted?: (productId: number) => void | Promise<void>;
		onUpdated?: (product: Product) => void;
	} = $props();

	let isDetailOpen = $state(false);
	let isEditOpen = $state(false);
	let isDeleteOpen = $state(false);

	function openDetail() {
		isDetailOpen = true;
	}

	function openEdit() {
		isEditOpen = true;
	}

	function openDelete() {
		isDeleteOpen = true;
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
					aria-label={`${t('common.action')}: ${product.name}`}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={openEdit}>
				<PencilIcon />
				{t('common.edit')}
			</DropdownMenu.Item>
			<DropdownMenu.Item variant="destructive" onSelect={openDelete}>
				<TrashIcon />
				{t('common.delete')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<ProductDetailDialog productId={product.id} bind:open={isDetailOpen} />
	<ProductEditDialog {product} bind:open={isEditOpen} {onUpdated} />
	<ProductDeleteAlertDialog {product} bind:open={isDeleteOpen} {onDeleted} />
</div>
