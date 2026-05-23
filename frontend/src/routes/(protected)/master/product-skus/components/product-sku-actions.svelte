<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { ProductSku } from '$lib/api/product-skus.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import ProductSkuDeleteAlertDialog from './product-sku-delete-alert-dialog.svelte';
	import ProductSkuDetailDialog from './product-sku-detail-dialog.svelte';
	import ProductSkuEditDialog from './product-sku-edit-dialog.svelte';

	let {
		productSku,
		onDeleted,
		onUpdated
	}: {
		productSku: ProductSku;
		onDeleted?: (productSkuId: number) => void | Promise<void>;
		onUpdated?: (productSku: ProductSku) => void;
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
					aria-label={`Open actions for ${productSku.sku}`}
					title="Actions"
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				Detail
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={openEdit}>
				<PencilIcon />
				Edit
			</DropdownMenu.Item>
			<DropdownMenu.Item variant="destructive" onSelect={openDelete}>
				<TrashIcon />
				Delete
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<ProductSkuDetailDialog productSkuId={productSku.id} bind:open={isDetailOpen} />
	<ProductSkuEditDialog {productSku} bind:open={isEditOpen} {onUpdated} />
	<ProductSkuDeleteAlertDialog {productSku} bind:open={isDeleteOpen} {onDeleted} />
</div>
