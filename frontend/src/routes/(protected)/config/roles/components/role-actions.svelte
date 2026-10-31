<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Permission, Role } from '$lib/api/roles.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import RoleDeleteAlertDialog from './role-delete-alert-dialog.svelte';
	import RoleDetailDialog from './role-detail-dialog.svelte';
	import RoleFormDialog from './role-form-dialog.svelte';

	let {
		role,
		permissions,
		onDeleted,
		onUpdated
	}: {
		role: Role;
		permissions: Permission[];
		onDeleted?: (roleId: number) => void | Promise<void>;
		onUpdated?: (role: Role) => void;
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
					aria-label={t('common.openActions').replace('{docNo}', role.name)}
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
			<DropdownMenu.Item
				variant="destructive"
				disabled={role.is_system}
				onSelect={() => {
					if (!role.is_system) isDeleteOpen = true;
				}}
			>
				<TrashIcon />
				{t('common.delete')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<RoleDetailDialog roleId={role.id} bind:open={isDetailOpen} />
	<RoleFormDialog {role} {permissions} bind:open={isEditOpen} onSaved={onUpdated} />
	<RoleDeleteAlertDialog {role} bind:open={isDeleteOpen} {onDeleted} />
</div>
