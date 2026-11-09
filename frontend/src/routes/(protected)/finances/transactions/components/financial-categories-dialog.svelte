<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import XIcon from '@lucide/svelte/icons/x';
	import {
		createFinancialCategory,
		deleteFinancialCategory,
		financialTransactionTypes,
		listFinancialCategories,
		updateFinancialCategory,
		type FinancialCategory,
		type FinancialTransactionType
	} from '$lib/api/finances.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { cn } from '$lib/utils.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	const categoryFormSchema = z.object({
		name: z.string().trim().min(1, 'Name is required.').max(100, 'Name is too long.'),
		category_type: z.enum(['income', 'expense']),
		description: z.string().max(500, 'Description is too long.'),
		is_active: z.enum(['true', 'false'])
	});

	type CategoryForm = z.infer<typeof categoryFormSchema>;

	let {
		open = $bindable(false),
		onChanged
	}: {
		open: boolean;
		onChanged?: () => void | Promise<void>;
	} = $props();

	let categories = $state<FinancialCategory[]>([]);
	let isLoading = $state(false);
	let loadErrorMessage = $state<string | null>(null);
	let editingId = $state<number | null>(null);
	let formData = $state<CategoryForm>(getDefaultFormData());
	let fieldErrors = $state<Record<string, string>>({});
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let deleteTarget = $state<FinancialCategory | null>(null);
	let isDeleting = $state(false);
	let deleteErrorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	const isEditing = $derived(editingId !== null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		deleteErrorMessage = null;
		deleteTarget = null;
		resetForm();
		void loadCategories();
	});

	async function loadCategories() {
		const requestId = ++latestRequestId;
		isLoading = true;
		loadErrorMessage = null;

		try {
			const result = await listFinancialCategories({ limit: 100 });
			if (requestId !== latestRequestId) return;
			categories = result.categories;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			categories = [];
			loadErrorMessage = getErrorMessage(error, t('finances.categories.errorLoad'));
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function startCreate() {
		editingId = null;
		resetForm();
	}

	function startEdit(category: FinancialCategory) {
		editingId = category.id;
		errorMessage = null;
		fieldErrors = {};
		formData = {
			name: category.name,
			category_type: category.category_type,
			description: category.description ?? '',
			is_active: category.is_active ? 'true' : 'false'
		};
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = categoryFormSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			if (editingId === null) {
				const created = await createFinancialCategory({
					name: result.data.name,
					category_type: result.data.category_type as FinancialTransactionType,
					description: emptyToNull(result.data.description),
					is_active: result.data.is_active === 'true'
				});
				toast.success(t('finances.categories.createdToast'), {
					description: `${created.name}`
				});
			} else {
				const updated = await updateFinancialCategory(editingId, {
					name: result.data.name,
					description: emptyToNull(result.data.description),
					is_active: result.data.is_active === 'true'
				});
				toast.success(t('finances.categories.updatedToast'), {
					description: `${updated.name}`
				});
			}
			resetForm();
			await loadCategories();
			await onChanged?.();
		} catch (error) {
			errorMessage = getErrorMessage(error, t('finances.categories.errorSave'));
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (!deleteTarget || isDeleting) return;

		isDeleting = true;
		deleteErrorMessage = null;

		try {
			await deleteFinancialCategory(deleteTarget.id);
			toast.success(t('finances.categories.deletedToast'), {
				description: `${deleteTarget.name}`
			});
			deleteTarget = null;
			if (editingId !== null) resetForm();
			await loadCategories();
			await onChanged?.();
		} catch (error) {
			deleteErrorMessage = getErrorMessage(error, t('finances.categories.errorDelete'));
		} finally {
			isDeleting = false;
		}
	}

	function resetForm() {
		editingId = null;
		fieldErrors = {};
		formData = getDefaultFormData();
	}

	function getDefaultFormData(): CategoryForm {
		return { name: '', category_type: 'expense', description: '', is_active: 'true' };
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getValidationErrors(error: z.ZodError) {
		const errors: Record<string, string> = {};
		for (const issue of error.issues) {
			const key = issue.path.map(String).join('.');
			if (!errors[key]) errors[key] = issue.message;
		}
		return errors;
	}

	function getTypeLabel(type: FinancialTransactionType) {
		return type === 'income'
			? t('finances.transactions.typeIncome')
			: t('finances.transactions.typeExpense');
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>{t('finances.categories.title')}</Dialog.Title>
			<Dialog.Description>{t('finances.categories.desc')}</Dialog.Description>
		</Dialog.Header>

		<div class="grid gap-6">
			{#if loadErrorMessage}
				<ErrorAlert title={t('finances.categories.errorLoad')} message={loadErrorMessage} />
			{:else if isLoading}
				<p class="text-sm text-muted-foreground">{t('common.loading')}</p>
			{:else if categories.length === 0}
				<p class="text-sm text-muted-foreground">{t('finances.categories.emptyDefault')}</p>
			{:else}
				<ul class="divide-y rounded-md border">
					{#each categories as category (category.id)}
						<li class="flex items-center gap-3 px-4 py-2.5">
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm font-medium">{category.name}</p>
								<div class="mt-1 flex items-center gap-2">
									<Badge variant={category.category_type === 'income' ? 'default' : 'secondary'}>
										{getTypeLabel(category.category_type)}
									</Badge>
									<StatusBadge active={category.is_active} />
								</div>
							</div>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								class="size-8"
								aria-label={`${t('common.edit')}: ${category.name}`}
								onclick={() => startEdit(category)}
							>
								<PencilIcon />
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								class="size-8 text-destructive"
								aria-label={`${t('common.delete')}: ${category.name}`}
								onclick={() => {
									deleteTarget = category;
									deleteErrorMessage = null;
								}}
							>
								<TrashIcon />
							</Button>
						</li>
					{/each}
				</ul>
			{/if}

			<form class="grid gap-4 rounded-md border p-4" onsubmit={handleSubmit}>
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-semibold">
						{isEditing ? t('finances.categories.editTitle') : t('finances.categories.addTitle')}
					</h3>
					{#if isEditing}
						<Button type="button" variant="ghost" size="sm" onclick={resetForm}>
							<XIcon />
							{t('common.cancel')}
						</Button>
					{/if}
				</div>

				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for="financial-category-name">{t('common.name')}</Label>
						<Input
							id="financial-category-name"
							placeholder={t('finances.categories.namePlaceholder')}
							disabled={isSubmitting}
							bind:value={formData.name}
						/>
						{#if fieldErrors.name}
							<p class="text-sm text-destructive">{fieldErrors.name}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="financial-category-type">{t('finances.transactions.typeLabel')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							value={formData.category_type}
							disabled={isSubmitting || isEditing}
							onValueChange={(value) => {
								if (value === 'income' || value === 'expense') formData.category_type = value;
							}}
						>
							<Select.Trigger id="financial-category-type" class="w-full">
								<Select.Value placeholder={t('finances.transactions.selectType')} />
							</Select.Trigger>
							<Select.Content>
								{#each financialTransactionTypes as transactionType (transactionType)}
									<Select.Item value={transactionType} label={getTypeLabel(transactionType)}>
										{getTypeLabel(transactionType)}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
						{#if isEditing}
							<p class="text-sm text-muted-foreground">{t('finances.categories.typeLockedHint')}</p>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<Label for="financial-category-description">
						{t('finances.transactions.descriptionLabel')}
					</Label>
					<Textarea
						id="financial-category-description"
						placeholder={t('finances.transactions.descriptionPlaceholder')}
						disabled={isSubmitting}
						bind:value={formData.description}
					/>
				</div>

				<div class="flex items-center justify-between gap-4">
					<div class="space-y-2">
						<Label for="financial-category-active">{t('common.status')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							value={formData.is_active}
							disabled={isSubmitting}
							onValueChange={(value) => {
								if (value === 'true' || value === 'false') formData.is_active = value;
							}}
						>
							<Select.Trigger id="financial-category-active" class="w-40">
								<Select.Value placeholder={t('common.selectStatus')} />
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="true" label={t('status.active')}
									>{t('status.active')}</Select.Item
								>
								<Select.Item value="false" label={t('status.inactive')}>
									{t('status.inactive')}
								</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>

					<Button type="submit" disabled={isSubmitting} class="mt-6">
						{#if isSubmitting}
							<Spinner />
						{:else if !isEditing}
							<PlusIcon />
						{/if}
						{isSubmitting
							? t('common.saving')
							: isEditing
								? t('finances.transactions.saveBtn')
								: t('finances.categories.createBtn')}
					</Button>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('finances.categories.errorSave')} message={errorMessage} />
				{/if}
			</form>
		</div>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root
	open={deleteTarget !== null}
	onOpenChange={(next) => !next && (deleteTarget = null)}
>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('finances.categories.deleteTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{t('finances.categories.deleteDesc').replace('{name}', deleteTarget?.name ?? '')}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if deleteErrorMessage}
			<ErrorAlert title={t('common.deleteFailed')} message={deleteErrorMessage} />
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
