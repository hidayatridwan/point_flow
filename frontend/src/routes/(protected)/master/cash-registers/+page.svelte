<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { onDestroy, onMount } from 'svelte';
	import {
		createCashRegister,
		deleteCashRegister,
		listCashRegisters,
		updateCashRegister,
		type CashRegister,
		type CashRegisterListParams,
		type CashRegisterPagination,
		type CashRegisterStatus
	} from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	type CashRegisterForm = {
		code: string;
		name: string;
		description: string;
		status: CashRegisterStatus;
	};

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: CashRegisterPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const emptyForm: CashRegisterForm = {
		code: '',
		name: '',
		description: '',
		status: 'active'
	};

	let cashRegisters = $state<CashRegister[]>([]);
	let pagination = $state<CashRegisterPagination>(defaultPagination);
	let isLoading = $state(true);
	let isSubmitting = $state(false);
	let isDeleting = $state(false);
	let errorMessage = $state<string | null>(null);
	let formErrorMessage = $state<string | null>(null);
	let deleteErrorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<CashRegisterListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isFormOpen = $state(false);
	let isDeleteOpen = $state(false);
	let editingRegister = $state<CashRegister | null>(null);
	let deletingRegister = $state<CashRegister | null>(null);
	let formData = $state<CashRegisterForm>({ ...emptyForm });
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadCashRegisters(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadCashRegisters(params: CashRegisterListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listCashRegisters(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadCashRegisters(query);
				return;
			}

			cashRegisters = result.cashRegisters;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error, t('master.cashRegisters.errorLoad'));
			cashRegisters = [];
		} finally {
			if (requestId === latestRequestId) isLoading = false;
		}
	}

	function openCreateDialog() {
		editingRegister = null;
		formData = { ...emptyForm };
		formErrorMessage = null;
		isFormOpen = true;
	}

	function openEditDialog(cashRegister: CashRegister) {
		editingRegister = cashRegister;
		formData = {
			code: cashRegister.code,
			name: cashRegister.name,
			description: cashRegister.description ?? '',
			status: cashRegister.status
		};
		formErrorMessage = null;
		isFormOpen = true;
	}

	function openDeleteDialog(cashRegister: CashRegister) {
		deletingRegister = cashRegister;
		deleteErrorMessage = null;
		isDeleteOpen = true;
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		const validationError = getFormValidationError();
		if (validationError) {
			formErrorMessage = validationError;
			return;
		}

		isSubmitting = true;
		formErrorMessage = null;

		try {
			const payload = {
				code: formData.code.trim(),
				name: formData.name.trim(),
				description: emptyToNull(formData.description),
				status: formData.status
			};
			const savedRegister = editingRegister
				? await updateCashRegister(editingRegister.id, payload)
				: await createCashRegister(payload);

			toast.success(
				editingRegister
					? t('master.cashRegisters.updatedToast')
					: t('master.cashRegisters.createdToast'),
				{
					description: `${savedRegister.code} ${t('common.createdToastDesc')}`
				}
			);
			isFormOpen = false;
			query = { ...query, page: editingRegister ? query.page : 1 };
			syncQueryUrl(query);
			await loadCashRegisters(query);
		} catch (error) {
			formErrorMessage = getErrorMessage(error, t('master.cashRegisters.errorSave'));
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (!deletingRegister || isDeleting) return;

		isDeleting = true;
		deleteErrorMessage = null;

		try {
			await deleteCashRegister(deletingRegister.id);
			toast.success(t('master.cashRegisters.deletedToast'), {
				description: `${deletingRegister.code} ${t('common.wasRemoved')}`
			});
			isDeleteOpen = false;
			await loadCashRegisters(query);
		} catch (error) {
			deleteErrorMessage = getErrorMessage(error, t('master.cashRegisters.errorDelete'));
		} finally {
			isDeleting = false;
		}
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;
		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadCashRegisters(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;
		query = { ...query, page };
		syncQueryUrl(query);
		void loadCashRegisters(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadCashRegisters(query);
		}, 350);
	}

	function parseQuery(searchParams: URLSearchParams): Required<CashRegisterListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			status: ['active', 'inactive']
		};
	}

	function syncQueryUrl(nextQuery: Required<CashRegisterListParams>) {
		const nextUrl = new URL(page.url);

		if (nextQuery.page === defaultPagination.page) nextUrl.searchParams.delete('page');
		else nextUrl.searchParams.set('page', String(nextQuery.page));

		if (nextQuery.limit === defaultPagination.pageSize) nextUrl.searchParams.delete('limit');
		else nextUrl.searchParams.set('limit', String(nextQuery.limit));

		if (nextQuery.search) nextUrl.searchParams.set('search', nextQuery.search);
		else nextUrl.searchParams.delete('search');

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function getFormValidationError() {
		if (!formData.code.trim()) return 'Code is required.';
		if (!formData.name.trim()) return 'Name is required.';
		return null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}
</script>

<svelte:head>
	<title>{t('master.cashRegisters.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('master.cashRegisters.title')}</Card.Title>
				<Card.Description>{t('master.cashRegisters.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('master.cashRegisters.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel={t('master.cashRegisters.searchAriaLabel')}
				placeholder={t('master.cashRegisters.searchPlaceholder')}
				onValueChange={updateSearchValue}
			/>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('master.cashRegisters.errorLoad')} message={errorMessage} />
			{:else}
				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>{t('master.cashRegisters.colCode')}</Table.Head>
									<Table.Head>{t('master.cashRegisters.colName')}</Table.Head>
									<Table.Head>{t('master.cashRegisters.colDescription')}</Table.Head>
									<Table.Head>{t('master.cashRegisters.colStatus')}</Table.Head>
									<Table.Head>{t('master.cashRegisters.colCreatedAt')}</Table.Head>
									<Table.Head class="w-28 text-center"
										>{t('master.cashRegisters.colAction')}</Table.Head
									>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#if isLoading}
									{#each Array.from({ length: 10 }).keys() as rowIndex (rowIndex)}
										<Table.Row>
											<Table.Cell><Skeleton class="h-5 w-24" /></Table.Cell>
											<Table.Cell><Skeleton class="h-5 w-40" /></Table.Cell>
											<Table.Cell><Skeleton class="h-5 w-52" /></Table.Cell>
											<Table.Cell><Skeleton class="h-5 w-20" /></Table.Cell>
											<Table.Cell><Skeleton class="h-5 w-32" /></Table.Cell>
											<Table.Cell class="text-center"
												><Skeleton class="mx-auto h-8 w-20" /></Table.Cell
											>
										</Table.Row>
									{/each}
								{:else if cashRegisters.length > 0}
									{#each cashRegisters as cashRegister (cashRegister.id)}
										<Table.Row>
											<Table.Cell class="font-medium">{cashRegister.code}</Table.Cell>
											<Table.Cell>{cashRegister.name}</Table.Cell>
											<Table.Cell>{cashRegister.description ?? '-'}</Table.Cell>
											<Table.Cell>
												<Badge variant="outline">{cashRegister.status}</Badge>
											</Table.Cell>
											<Table.Cell>{formatDateTime(cashRegister.created_at)}</Table.Cell>
											<Table.Cell>
												<div class="flex justify-center gap-1">
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														class="size-8"
														title={t('common.edit')}
														onclick={() => openEditDialog(cashRegister)}
													>
														<PencilIcon />
														<span class="sr-only">{t('common.edit')} {cashRegister.code}</span>
													</Button>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														class="size-8 text-destructive hover:bg-destructive/10 hover:text-destructive"
														title={t('common.delete')}
														onclick={() => openDeleteDialog(cashRegister)}
													>
														<TrashIcon />
														<span class="sr-only">{t('common.delete')} {cashRegister.code}</span>
													</Button>
												</div>
											</Table.Cell>
										</Table.Row>
									{/each}
								{:else}
									<Table.Row>
										<Table.Cell colspan={6} class="h-24 text-center text-muted-foreground">
											{query.search
												? t('master.cashRegisters.emptySearch')
												: t('master.cashRegisters.emptyDefault')}
										</Table.Cell>
									</Table.Row>
								{/if}
							</Table.Body>
						</Table.Root>
					</div>
				</div>
			{/if}
		</Card.Content>
		{#if !isLoading && !errorMessage}
			<Card.Footer>
				<DataPagination
					{pagination}
					{pageSizeOptions}
					onPageChange={updatePage}
					onPageSizeChange={updatePageSize}
				/>
			</Card.Footer>
		{/if}
	</Card.Root>
</div>

<Dialog.Root bind:open={isFormOpen}>
	<Dialog.Content class="p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title
					>{editingRegister
						? t('master.cashRegisters.editTitle')
						: t('master.cashRegisters.addTitle')}</Dialog.Title
				>
				<Dialog.Description>
					{t('master.cashRegisters.formDesc')}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<div class="grid gap-2">
					<Label for="cash-register-code">{t('master.cashRegisters.code')}</Label>
					<Input
						id="cash-register-code"
						placeholder="REG-01"
						disabled={isSubmitting}
						bind:value={formData.code}
					/>
				</div>
				<div class="grid gap-2">
					<Label for="cash-register-name">{t('common.name')}</Label>
					<Input
						id="cash-register-name"
						placeholder={t('master.cashRegisters.addName')}
						disabled={isSubmitting}
						bind:value={formData.name}
					/>
				</div>
				<div class="grid gap-2">
					<Label for="cash-register-description">{t('common.description')}</Label>
					<Textarea
						id="cash-register-description"
						placeholder={t('master.cashRegisters.addDescription')}
						disabled={isSubmitting}
						bind:value={formData.description}
					/>
				</div>
				<div class="grid gap-2">
					<Label>{t('common.status')}</Label>
					<Select.Root
						allowDeselect={false}
						type="single"
						value={formData.status}
						disabled={isSubmitting}
						onValueChange={(value) => {
							if (value === 'active' || value === 'inactive') formData.status = value;
						}}
					>
						<Select.Trigger class="w-full">
							<Select.Value placeholder={t('common.selectStatus')} />
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="active" label={t('status.active')}
								>{t('status.active')}</Select.Item
							>
							<Select.Item value="inactive" label={t('status.inactive')}
								>{t('status.inactive')}</Select.Item
							>
						</Select.Content>
					</Select.Root>
				</div>

				{#if formErrorMessage}
					<ErrorAlert title={t('common.saveFailed')} message={formErrorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('master.cashRegisters.cancelInlineBtn')}
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting}>
					{isSubmitting
						? t('master.cashRegisters.savingRegister')
						: t('master.cashRegisters.saveRegisterBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

{#if deletingRegister}
	<AlertDialog.Root bind:open={isDeleteOpen}>
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title>{t('master.cashRegisters.deleteTitle')}</AlertDialog.Title>
				<AlertDialog.Description>
					{t('master.cashRegisters.deleteDesc').replace('{code}', deletingRegister.code)}
				</AlertDialog.Description>
			</AlertDialog.Header>

			{#if deleteErrorMessage}
				<ErrorAlert title={t('master.cashRegisters.deleteFailed')} message={deleteErrorMessage} />
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
{/if}
