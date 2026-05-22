<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Unit } from '$lib/api/units.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import UnitDeleteAlertDialog from './unit-delete-alert-dialog.svelte';
	import UnitDetailDialog from './unit-detail-dialog.svelte';
	import UnitEditDialog from './unit-edit-dialog.svelte';

	let {
		unit,
		onDeleted,
		onUpdated
	}: {
		unit: Unit;
		onDeleted?: (unitId: number) => void | Promise<void>;
		onUpdated?: (unit: Unit) => void;
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
					aria-label={`Open actions for ${unit.name}`}
					title="Actions"
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={() => (isDetailOpen = true)}>
				<EyeIcon />
				Detail
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={() => (isEditOpen = true)}>
				<PencilIcon />
				Edit
			</DropdownMenu.Item>
			<DropdownMenu.Item variant="destructive" onSelect={() => (isDeleteOpen = true)}>
				<TrashIcon />
				Delete
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<UnitDetailDialog unitId={unit.id} bind:open={isDetailOpen} />
	<UnitEditDialog {unit} bind:open={isEditOpen} {onUpdated} />
	<UnitDeleteAlertDialog {unit} bind:open={isDeleteOpen} {onDeleted} />
</div>
