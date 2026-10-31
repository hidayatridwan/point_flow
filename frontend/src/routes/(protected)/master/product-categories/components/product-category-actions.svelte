<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { ProductCategory } from '$lib/api/product-categories.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import ProductCategoryDeleteAlertDialog from './product-category-delete-alert-dialog.svelte';
	import ProductCategoryDetailDialog from './product-category-detail-dialog.svelte';
	import ProductCategoryEditDialog from './product-category-edit-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		category,
		onDeleted,
		onUpdated
	}: {
		category: ProductCategory;
		onDeleted?: (categoryId: number) => void | Promise<void>;
		onUpdated?: (category: ProductCategory) => void;
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
					aria-label={`${t('common.action')}: ${category.name}`}
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

	<ProductCategoryDetailDialog categoryId={category.id} bind:open={isDetailOpen} />
	<ProductCategoryEditDialog {category} bind:open={isEditOpen} {onUpdated} />
	<ProductCategoryDeleteAlertDialog {category} bind:open={isDeleteOpen} {onDeleted} />
</div>
