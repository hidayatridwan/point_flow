<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Employee } from '$lib/api/employees.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import EmployeeDeleteAlertDialog from './employee-delete-alert-dialog.svelte';
	import EmployeeDetailDialog from './employee-detail-dialog.svelte';
	import EmployeeEditDialog from './employee-edit-dialog.svelte';

	let {
		employee,
		onDeleted,
		onUpdated
	}: {
		employee: Employee;
		onDeleted?: (employeeId: number) => void | Promise<void>;
		onUpdated?: (employee: Employee) => void;
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
					aria-label={`Open actions for ${employee.name}`}
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

	<EmployeeDetailDialog employeeId={employee.id} bind:open={isDetailOpen} />
	<EmployeeEditDialog {employee} bind:open={isEditOpen} {onUpdated} />
	<EmployeeDeleteAlertDialog {employee} bind:open={isDeleteOpen} {onDeleted} />
</div>
