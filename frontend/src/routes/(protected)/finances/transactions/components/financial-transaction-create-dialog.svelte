<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import { parseDate, type DateValue } from '@internationalized/date';
	import {
		createFinancialTransaction,
		financialTransactionTypes,
		listFinancialCategories,
		type FinancialCategory,
		type FinancialTransaction,
		type FinancialTransactionType
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
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	const createTransactionSchema = z.object({
		transaction_type: z.enum(['income', 'expense']),
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

	type CreateTransactionForm = z.infer<typeof createTransactionSchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (transaction: FinancialTransaction) => void | Promise<void>;
	} = $props();

	let categories = $state<FinancialCategory[]>([]);
	let isLoadingOptions = $state(false);
	let optionsErrorMessage = $state<string | null>(null);
	let occurredDateValue = $state<DateValue | undefined>(dateStringToDateValue(getTodayDate()));
	let isOccurredDateOpen = $state(false);
	let formData = $state<CreateTransactionForm>(getDefaultFormData());
	let fieldErrors = $state<Record<string, string>>({});
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const activeCategories = $derived(categories.filter((category) => category.is_active));
	const filteredCategories = $derived(
		activeCategories.filter((category) => category.category_type === formData.transaction_type)
	);

	// Label source untuk Select.Root: tanpa ini, trigger menampilkan nilai mentah
	// ("expense"/"true"/ID) sampai dropdown dibuka pertama kali (bits-ui lazy items).
	const typeItems = $derived([
		{ value: 'income', label: getTypeLabel('income') },
		{ value: 'expense', label: getTypeLabel('expense') }
	]);
	const categoryItems = $derived(
		filteredCategories.map((category) => ({ value: String(category.id), label: category.name }))
	);
	const cashItems = $derived([
		{ value: 'true', label: t('finances.transactions.cashYes') },
		{ value: 'false', label: t('finances.transactions.cashNo') }
	]);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		fieldErrors = {};
		formData = getDefaultFormData();
		occurredDateValue = dateStringToDateValue(getTodayDate());
		isOccurredDateOpen = false;
		void loadCategories();
	});

	$effect(() => {
		formData.occurred_date = occurredDateValue?.toString() ?? '';
	});

	$effect(() => {
		const stillValid = filteredCategories.some(
			(category) => String(category.id) === formData.financial_category_id
		);
		if (!stillValid) formData.financial_category_id = '';
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
		const result = createTransactionSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		const category = filteredCategories.find(
			(item) => String(item.id) === result.data.financial_category_id
		);
		if (!category) {
			fieldErrors = { financial_category_id: t('finances.transactions.validationCategory') };
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const created = await createFinancialTransaction({
				transaction_type: result.data.transaction_type as FinancialTransactionType,
				financial_category_id: category.id,
				occurred_at: toUtcMidnight(result.data.occurred_date),
				amount: Number(result.data.amount),
				is_cash: result.data.is_cash === 'true',
				description: emptyToNull(result.data.description)
			});
			await onCreated?.(created);
			toast.success(t('finances.transactions.createdToast'), {
				description: `${created.transaction_no}`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('finances.transactions.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	function getDefaultFormData(): CreateTransactionForm {
		return {
			transaction_type: 'expense',
			financial_category_id: '',
			occurred_date: getTodayDate(),
			amount: '',
			is_cash: 'true',
			description: ''
		};
	}

	function getTodayDate() {
		const date = new Date();
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
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
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header>
				<Dialog.Title>{t('finances.transactions.addTitle')}</Dialog.Title>
				<Dialog.Description>{t('finances.transactions.addDesc')}</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for="financial-transaction-type">{t('finances.transactions.typeLabel')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							items={typeItems}
							value={formData.transaction_type}
							disabled={isSubmitting}
							onValueChange={(value) => {
								if (value === 'income' || value === 'expense') {
									formData.transaction_type = value;
								}
							}}
						>
							<Select.Trigger id="financial-transaction-type" class="w-full">
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
					</div>

					<div class="space-y-2">
						<Label for="financial-transaction-date">{t('finances.transactions.dateLabel')}</Label>
						<Popover.Root bind:open={isOccurredDateOpen}>
							<Popover.Trigger
								id="financial-transaction-date"
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
				</div>

				<div class="space-y-2">
					<Label for="financial-transaction-category">
						{t('finances.transactions.categoryLabel')}
					</Label>
					<Select.Root
						allowDeselect={false}
						type="single"
						items={categoryItems}
						value={formData.financial_category_id}
						disabled={isSubmitting || isLoadingOptions}
						onValueChange={(value) => {
							if (typeof value === 'string') formData.financial_category_id = value;
						}}
					>
						<Select.Trigger id="financial-transaction-category" class="w-full">
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

				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for="financial-transaction-amount">
							{t('finances.transactions.amountLabel')}
						</Label>
						<Input
							id="financial-transaction-amount"
							inputmode="decimal"
							placeholder="150000"
							disabled={isSubmitting}
							bind:value={formData.amount}
						/>
						{#if getFieldError('amount')}
							<p class="text-sm text-destructive">{getFieldError('amount')}</p>
						{:else if Number.isFinite(Number(formData.amount)) && formData.amount.trim() !== ''}
							<p class="text-sm text-muted-foreground">{formatIdr(Number(formData.amount))}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="financial-transaction-cash">{t('finances.transactions.cashLabel')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							items={cashItems}
							value={formData.is_cash}
							disabled={isSubmitting}
							onValueChange={(value) => {
								if (value === 'true' || value === 'false') formData.is_cash = value;
							}}
						>
							<Select.Trigger id="financial-transaction-cash" class="w-full">
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
				</div>

				<div class="space-y-2">
					<Label for="financial-transaction-description">
						{t('finances.transactions.descriptionLabel')}
					</Label>
					<Textarea
						id="financial-transaction-description"
						placeholder={t('finances.transactions.descriptionPlaceholder')}
						disabled={isSubmitting}
						bind:value={formData.description}
					/>
					{#if getFieldError('description')}
						<p class="text-sm text-destructive">{getFieldError('description')}</p>
					{/if}
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('finances.transactions.errorCreate')} message={errorMessage} />
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
					{isSubmitting
						? t('finances.transactions.creating')
						: t('finances.transactions.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
