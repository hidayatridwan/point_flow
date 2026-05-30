<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteEmployee, type Employee } from '$lib/api/employees.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		employee,
		open = $bindable(false),
		onDeleted
	}: {
		employee: Employee;
		open: boolean;
		onDeleted?: (employeeId: number) => void | Promise<void>;
	} = $props();

	let isDeleting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (isDeleting) return;

		isDeleting = true;
		errorMessage = null;

		try {
			await deleteEmployee(employee.id);
			await onDeleted?.(employee.id);
			toast.success(t('master.employees.deletedToast'), {
				description: `${employee.name} ${t('common.wasRemoved')}`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isDeleting = false;
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.employees.errorDelete');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('master.employees.deleteTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				<span class="font-medium text-foreground">{employee.name}</span> — {t('master.employees.deleteDesc').replace('{name}', '')}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.deleteFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" disabled={isDeleting} onclick={handleDelete}>
				<TrashIcon />
				{isDeleting ? t('common.deleting') : t('common.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
