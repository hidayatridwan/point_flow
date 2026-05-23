<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { listEmployees, type Employee } from '$lib/api/employees.js';
	import { createPayrollSlip, type PayrollSlip } from '$lib/api/payroll.js';
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
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	type PayrollSlipForm = {
		employee_id: string;
		period_start: string;
		period_end: string;
		bonus: string;
		deduction: string;
	};

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	const requiredIdSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.min(1, `${label} is required.`)
				.refine((value) => Number.isInteger(Number(value)) && Number(value) > 0, {
					message: `${label} must be selected.`
				})
		);

	const requiredDateSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.min(1, `${label} is required.`)
				.refine(isValidDateString, {
					message: `${label} must be a valid date.`
				})
		);

	const nonNegativeMoneySchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.refine((value) => value === '' || Number.isFinite(Number(value)), {
					message: `${label} must be a valid number.`
				})
				.refine((value) => parseMoneyInput(value) >= 0, {
					message: `${label} must be zero or greater.`
				})
		);

	const createPayrollSlipSchema = z
		.object({
			employee_id: requiredIdSchema('Employee'),
			period_start: requiredDateSchema('Period start'),
			period_end: requiredDateSchema('Period end'),
			bonus: nonNegativeMoneySchema('Bonus'),
			deduction: nonNegativeMoneySchema('Deduction')
		})
		.superRefine((data, context) => {
			if (data.period_start && data.period_end && data.period_start > data.period_end) {
				context.addIssue({
					code: 'custom',
					path: ['period_end'],
					message: 'Period end must be on or after period start.'
				});
			}
		});

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (payrollSlip: PayrollSlip) => void | Promise<void>;
	} = $props();

	let employees = $state<Employee[]>([]);
	let periodStartValue = $state<DateValue | undefined>(
		dateStringToDateValue(getCurrentMonthStartDate())
	);
	let periodEndValue = $state<DateValue | undefined>(dateStringToDateValue(getTodayDate()));
	let isPeriodStartOpen = $state(false);
	let isPeriodEndOpen = $state(false);
	let formData = $state<PayrollSlipForm>(getDefaultFormData());
	let fieldErrors = $state<Record<string, string>>({});
	let isLoadingOptions = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const selectedEmployee = $derived(
		employees.find((employee) => String(employee.id) === formData.employee_id) ?? null
	);
	const previewNetSalary = $derived(
		(selectedEmployee?.base_salary ?? 0) +
			parseMoneyInput(formData.bonus) -
			parseMoneyInput(formData.deduction)
	);

	$effect(() => {
		if (!open) return;
		resetFormData();
		fieldErrors = {};
		errorMessage = null;
		optionsErrorMessage = null;
		void loadEmployees();
	});

	$effect(() => {
		const periodStart = dateValueToDateString(periodStartValue);
		if (formData.period_start !== periodStart) {
			formData.period_start = periodStart;
		}
		if (periodStartValue) {
			isPeriodStartOpen = false;
		}
	});

	$effect(() => {
		const periodEnd = dateValueToDateString(periodEndValue);
		if (formData.period_end !== periodEnd) {
			formData.period_end = periodEnd;
		}
		if (periodEndValue) {
			isPeriodEndOpen = false;
		}
	});

	async function loadEmployees() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const result = await listEmployees({ limit: 100 });
			if (requestId !== latestOptionsRequestId) return;
			employees = result.employees.filter((employee) => employee.is_active);
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			employees = [];
			optionsErrorMessage = getErrorMessage(error, 'Failed to load employee options.');
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = createPayrollSlipSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		const employee = employees.find((item) => String(item.id) === result.data.employee_id);
		if (!employee) {
			fieldErrors = { employee_id: 'Employee must be selected.' };
			return;
		}

		const bonus = parseMoneyInput(result.data.bonus);
		const deduction = parseMoneyInput(result.data.deduction);
		const netSalary = employee.base_salary + bonus - deduction;

		if (netSalary < 0) {
			fieldErrors = { deduction: 'Net salary must not be negative.' };
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const createdPayrollSlip = await createPayrollSlip({
				employee_id: Number(result.data.employee_id),
				period_start: result.data.period_start,
				period_end: result.data.period_end,
				bonus,
				deduction
			});
			await onCreated?.(createdPayrollSlip);
			toast.success('Payroll slip created.', {
				description: `${createdPayrollSlip.slip_no} was created.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, 'Failed to create payroll slip.');
		} finally {
			isSubmitting = false;
		}
	}

	function resetFormData() {
		const periodStart = getCurrentMonthStartDate();
		const periodEnd = getTodayDate();
		periodStartValue = dateStringToDateValue(periodStart);
		periodEndValue = dateStringToDateValue(periodEnd);
		isPeriodStartOpen = false;
		isPeriodEndOpen = false;
		formData = getDefaultFormData(periodStart, periodEnd);
	}

	function getDefaultFormData(
		periodStart = getCurrentMonthStartDate(),
		periodEnd = getTodayDate()
	) {
		return {
			employee_id: '',
			period_start: periodStart,
			period_end: periodEnd,
			bonus: '0',
			deduction: '0'
		};
	}

	function getEmployeeLabel(employee: Employee) {
		return `${employee.name} - ${employee.position}`;
	}

	function getCurrentMonthStartDate() {
		const date = new Date();
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		return `${year}-${month}-01`;
	}

	function getTodayDate() {
		const date = new Date();
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	function isValidDateString(value: string) {
		if (!value) return false;

		try {
			parseDate(value);
			return true;
		} catch {
			return false;
		}
	}

	function dateStringToDateValue(value: string): DateValue | undefined {
		if (!value) return undefined;

		try {
			return parseDate(value);
		} catch {
			return undefined;
		}
	}

	function dateValueToDateString(value: DateValue | undefined) {
		return value?.toString() ?? '';
	}

	function parseMoneyInput(value: string) {
		const number = Number(value || '0');
		return Number.isFinite(number) ? number : 0;
	}

	function getValidationErrors(error: z.ZodError) {
		const errors: Record<string, string> = {};
		for (const issue of error.issues) {
			const key = issue.path.map(String).join('.');
			if (!errors[key]) errors[key] = issue.message;
		}
		return errors;
	}

	function getFieldError(path: string) {
		return fieldErrors[path] ?? null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-hidden p-0 sm:max-w-lg">
		<form class="grid max-h-[90vh] grid-rows-[auto_minmax(0,1fr)_auto]" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Add Payroll Slip</Dialog.Title>
				<Dialog.Description>
					Create a payroll slip from an active employee base salary.
				</Dialog.Description>
			</Dialog.Header>

			<div class="overflow-y-auto px-6 py-5">
				<div class="grid gap-5">
					<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2">
						<div class="space-y-2 md:col-span-2">
							<Label for="payroll-slip-employee" class="whitespace-nowrap">Employee</Label>
							<Select.Root
								allowDeselect={false}
								type="single"
								value={formData.employee_id}
								disabled={isSubmitting || isLoadingOptions}
								onValueChange={(value) => {
									if (typeof value === 'string') {
										formData.employee_id = value;
									}
								}}
							>
								<Select.Trigger id="payroll-slip-employee" class="w-full">
									<Select.Value placeholder="Select employee" />
								</Select.Trigger>
								<Select.Content>
									{#each employees as employee (employee.id)}
										<Select.Item value={String(employee.id)} label={getEmployeeLabel(employee)}>
											{getEmployeeLabel(employee)}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
							{#if getFieldError('employee_id')}
								<p class="text-sm text-destructive">{getFieldError('employee_id')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="payroll-slip-period-start" class="whitespace-nowrap">Period Start</Label>
							<Popover.Root bind:open={isPeriodStartOpen}>
								<Popover.Trigger
									id="payroll-slip-period-start"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.period_start && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.period_start
											? formatDate(formData.period_start)
											: 'Select period start'}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={periodStartValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('period_start')}
								<p class="text-sm text-destructive">{getFieldError('period_start')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="payroll-slip-period-end" class="whitespace-nowrap">Period End</Label>
							<Popover.Root bind:open={isPeriodEndOpen}>
								<Popover.Trigger
									id="payroll-slip-period-end"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.period_end && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.period_end ? formatDate(formData.period_end) : 'Select period end'}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={periodEndValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('period_end')}
								<p class="text-sm text-destructive">{getFieldError('period_end')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="payroll-slip-bonus" class="whitespace-nowrap">Bonus</Label>
							<Input
								id="payroll-slip-bonus"
								type="number"
								min="0"
								step="1000"
								placeholder="0"
								disabled={isSubmitting}
								bind:value={formData.bonus}
							/>
							{#if getFieldError('bonus')}
								<p class="text-sm text-destructive">{getFieldError('bonus')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="payroll-slip-deduction" class="whitespace-nowrap">Deduction</Label>
							<Input
								id="payroll-slip-deduction"
								type="number"
								min="0"
								step="1000"
								placeholder="0"
								disabled={isSubmitting}
								bind:value={formData.deduction}
							/>
							{#if getFieldError('deduction')}
								<p class="text-sm text-destructive">{getFieldError('deduction')}</p>
							{/if}
						</div>
					</div>

					<div class="grid gap-3 rounded-lg border bg-muted/20 p-4 sm:grid-cols-2">
						<div>
							<p class="text-xs font-medium text-muted-foreground">Base Salary</p>
							<p class="mt-1 font-semibold">
								{selectedEmployee ? formatIdr(selectedEmployee.base_salary) : '-'}
							</p>
						</div>
						<div>
							<p class="text-xs font-medium text-muted-foreground">Bonus</p>
							<p class="mt-1 font-semibold">{formatIdr(parseMoneyInput(formData.bonus))}</p>
						</div>
						<div>
							<p class="text-xs font-medium text-muted-foreground">Deduction</p>
							<p class="mt-1 font-semibold">{formatIdr(parseMoneyInput(formData.deduction))}</p>
						</div>
						<div>
							<p class="text-xs font-medium text-muted-foreground">Net Salary Preview</p>
							<p class="mt-1 font-semibold">
								{selectedEmployee ? formatIdr(previewNetSalary) : '-'}
							</p>
						</div>
					</div>

					<div class="grid gap-3">
						{#if optionsErrorMessage}
							<ErrorAlert title="Failed to load options." message={optionsErrorMessage} />
						{/if}

						{#if errorMessage}
							<ErrorAlert title="Failed to create payroll slip." message={errorMessage} />
						{/if}
					</div>
				</div>
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					Cancel
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting || isLoadingOptions}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? 'Creating...' : 'Create payroll slip'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
