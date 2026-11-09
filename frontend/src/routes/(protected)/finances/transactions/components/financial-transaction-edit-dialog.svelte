<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import { parseDate, type DateValue } from '@internationalized/date';
	import {
		listFinancialCategories,
		updateFinancialTransaction,
		type FinancialCategory,
		type FinancialTransaction
	} from '$lib/api/finances.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import Calendar from '$lib/components/ui/calendar/calendar.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	const editTransactionSchema = z.object({
		financial_category_id: z.string().min(1, 'Category is required.'),
		occurred_date: z.string().min(1, 'Date is required.'),
		amount: z
			.string()
			.trim()
			.refine((value) => value !== '' && Number.isFinite(Number(value)), {
				message: 'Amount must be a valid number.'
			})
			.refine((value) => Number(value) > 0, {
				message: 'Amount must be greater than 0.'
			}),
		is_cash: z.enum(['true', 'false']),
		description: z.string().max(500, 'Description must be at most 500 characters.')
	});

	type EditTransactionForm = z.infer<typeof editTransactionSchema>;

	let {
		transaction,
		open = $bindable(false),
		onUpdated
	}: {
		transaction: FinancialTransaction;
		open: boolean;
		onUpdated?: (transaction: FinancialTransaction) => void;
	} = $props();

	let categories = $state<FinancialCategory[]>([]);
	let isLoadingOptions = $state(false);
	let optionsErrorMessage = $state<string | null>(null);
	let occurredDateValue = $state<DateValue | undefined>(undefined);
	let isOccurredDateOpen = $state(false);
	let formData = $state<EditTransactionForm>(getDefaultFormData());
	let fieldErrors = $state<Record<string, string>>({});
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const filteredCategories = $derived(
		categories.filter(
			(category) => category.is_active && category.category_type === transaction.transaction_type
		)
	);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		fieldErrors = {};
		formData = getInitialFormData(transaction);
		occurredDateValue = dateStringToDateValue(formData.occurred_date);
		isOccurredDateOpen = false;
		void loadCategories();
	});

	$effect(() => {
		formData.occurred_date = occurredDateValue?.toString() ?? '';
	});

	async function loadCategories() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const result = await listFinancialCategories({ limit: 100 });
			if (requestId !== latestOptionsRequestId) return;
			categories = result.categories;
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			categories = [];
			optionsErrorMessage = getErrorMessage(error, t('finances.transactions.errorLoadCategories'));
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = editTransactionSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		const category = filteredCategories.find(
			(item) => String(item.id) === result.data.financial_category_id
		);
		if (
			!category &&
			result.data.financial_category_id !== String(transaction.financial_category_id)
		) {
			fieldErrors = { financial_category_id: t('finances.transactions.validationCategory') };
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const updated = await updateFinancialTransaction(transaction.id, {
				financial_category_id: Number(result.data.financial_category_id),
				occurred_at: toUtcMidnight(result.data.occurred_date),
				amount: Number(result.data.amount),
				is_cash: result.data.is_cash === 'true',
				description: emptyToNull(result.data.description)
			});
			await onUpdated?.(updated);
			toast.success(t('finances.transactions.updatedToast'), {
				description: `${updated.transaction_no}`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('finances.transactions.errorSave'));
		} finally {
			isSubmitting = false;
		}
	}

	function getDefaultFormData(): EditTransactionForm {
		return {
			financial_category_id: '',
			occurred_date: '',
			amount: '',
			is_cash: 'true',
			description: ''
		};
	}

	function getInitialFormData(current: FinancialTransaction): EditTransactionForm {
		return {
			financial_category_id: String(current.financial_category_id),
			occurred_date: current.occurred_at.slice(0, 10),
			amount: String(current.amount),
			is_cash: current.is_cash ? 'true' : 'false',
			description: current.description ?? ''
		};
	}

	function dateStringToDateValue(value: string): DateValue | undefined {
		if (!value) return undefined;
		try {
			return parseDate(value);
		} catch {
			return undefined;
		}
	}

	function toUtcMidnight(value: string) {
		return `${value}T00:00:00Z`;
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

	function getFieldError(name: string) {
		return fieldErrors[name];
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header>
				<Dialog.Title>{t('finances.transactions.editTitle')}</Dialog.Title>
				<Dialog.Description>
					{t('finances.transactions.editDesc')
						.replace('{no}', transaction.transaction_no)
						.replace('{date}', formatDateTime(transaction.occurred_at))}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for="financial-transaction-edit-date">
							{t('finances.transactions.dateLabel')}
						</Label>
						<Popover.Root bind:open={isOccurredDateOpen}>
							<Popover.Trigger
								id="financial-transaction-edit-date"
								class={cn(
									buttonVariants({ variant: 'outline' }),
									'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
									!formData.occurred_date && 'text-muted-foreground'
								)}
								disabled={isSubmitting}
							>
								<CalendarIcon class="size-4" />
								<span class="truncate">
									{formData.occurred_date
										? formatDate(formData.occurred_date)
										: t('finances.transactions.selectDate')}
								</span>
							</Popover.Trigger>
							<Popover.Content align="start" class="w-auto p-0">
								<Calendar
									type="single"
									bind:value={occurredDateValue}
									class="rounded-md border-0 shadow-none"
								/>
							</Popover.Content>
						</Popover.Root>
						{#if getFieldError('occurred_date')}
							<p class="text-sm text-destructive">{getFieldError('occurred_date')}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="financial-transaction-edit-amount">
							{t('finances.transactions.amountLabel')}
						</Label>
						<Input
							id="financial-transaction-edit-amount"
							inputmode="decimal"
							disabled={isSubmitting}
							bind:value={formData.amount}
						/>
						{#if getFieldError('amount')}
							<p class="text-sm text-destructive">{getFieldError('amount')}</p>
						{:else if Number.isFinite(Number(formData.amount)) && formData.amount.trim() !== ''}
							<p class="text-sm text-muted-foreground">{formatIdr(Number(formData.amount))}</p>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<Label for="financial-transaction-edit-category">
						{t('finances.transactions.categoryLabel')}
					</Label>
					<Select.Root
						allowDeselect={false}
						type="single"
						value={formData.financial_category_id}
						disabled={isSubmitting || isLoadingOptions}
						onValueChange={(value) => {
							if (typeof value === 'string') formData.financial_category_id = value;
						}}
					>
						<Select.Trigger id="financial-transaction-edit-category" class="w-full">
							<Select.Value placeholder={t('finances.transactions.selectCategoryPlaceholder')} />
						</Select.Trigger>
						<Select.Content>
							{#each filteredCategories as category (category.id)}
								<Select.Item value={String(category.id)} label={category.name}>
									{category.name}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					{#if getFieldError('financial_category_id')}
						<p class="text-sm text-destructive">{getFieldError('financial_category_id')}</p>
					{/if}
					{#if optionsErrorMessage}
						<p class="text-sm text-destructive">{optionsErrorMessage}</p>
					{/if}
				</div>

				<div class="space-y-2">
					<Label for="financial-transaction-edit-cash">{t('finances.transactions.cashLabel')}</Label
					>
					<Select.Root
						allowDeselect={false}
						type="single"
						value={formData.is_cash}
						disabled={isSubmitting}
						onValueChange={(value) => {
							if (value === 'true' || value === 'false') formData.is_cash = value;
						}}
					>
						<Select.Trigger id="financial-transaction-edit-cash" class="w-full">
							<Select.Value placeholder={t('finances.transactions.selectCash')} />
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="true" label={t('finances.transactions.cashYes')}>
								{t('finances.transactions.cashYes')}
							</Select.Item>
							<Select.Item value="false" label={t('finances.transactions.cashNo')}>
								{t('finances.transactions.cashNo')}
							</Select.Item>
						</Select.Content>
					</Select.Root>
					<p class="text-sm text-muted-foreground">{t('finances.transactions.cashHint')}</p>
				</div>

				<div class="space-y-2">
					<Label for="financial-transaction-edit-description">
						{t('finances.transactions.descriptionLabel')}
					</Label>
					<Textarea
						id="financial-transaction-edit-description"
						placeholder={t('finances.transactions.descriptionPlaceholder')}
						disabled={isSubmitting}
						bind:value={formData.description}
					/>
					{#if getFieldError('description')}
						<p class="text-sm text-destructive">{getFieldError('description')}</p>
					{/if}
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('finances.transactions.errorSave')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('common.cancel')}
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting || isLoadingOptions}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? t('finances.transactions.saving') : t('finances.transactions.saveBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
