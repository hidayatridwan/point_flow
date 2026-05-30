<script lang="ts">
	import LockKeyholeIcon from '@lucide/svelte/icons/lock-keyhole';
	import MailIcon from '@lucide/svelte/icons/mail';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Field, FieldGroup, FieldLabel } from '$lib/components/ui/field/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';
	import { t } from '$lib/i18n/locale.svelte.js';

	type LoginCredentials = {
		email: string;
		password: string;
	};

	let {
		class: className,
		isSubmitting = false,
		errorMessage = null,
		onSubmit,
		...restProps
	}: HTMLAttributes<HTMLDivElement> & {
		isSubmitting?: boolean;
		errorMessage?: string | null;
		onSubmit?: (credentials: LoginCredentials) => void | Promise<void>;
	} = $props();

	const id = $props.id();
	let email = $state('');
	let password = $state('');
	let validationMessage = $state<string | null>(null);
	const visibleError = $derived(validationMessage ?? errorMessage);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		validationMessage = null;

		const normalizedEmail = email.trim();
		if (!normalizedEmail || !password) {
			validationMessage = t('login.validationRequired');
			return;
		}

		await onSubmit?.({
			email: normalizedEmail,
			password
		});
	}
</script>

<div class={cn('flex flex-col gap-6', className)} {...restProps}>
	<Card.Root>
		<Card.Header class="text-center">
			<Card.Title class="text-xl">{t('login.title')}</Card.Title>
			<Card.Description>{t('login.subtitle')}</Card.Description>
		</Card.Header>
		<Card.Content>
			<form onsubmit={handleSubmit}>
				<FieldGroup>
					<Field>
						<FieldLabel for="email-{id}">{t('login.email')}</FieldLabel>
						<div class="relative">
							<MailIcon
								class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
							/>
							<Input
								id="email-{id}"
								type="email"
								placeholder="admin@example.com"
								autocomplete="email"
								class="pl-8"
								disabled={isSubmitting}
								bind:value={email}
							/>
						</div>
					</Field>
					<Field>
						<div class="flex items-center justify-between gap-3">
							<FieldLabel for="password-{id}">{t('login.password')}</FieldLabel>
							<a
								href="mailto:support@example.com?subject=Password%20reset%20request"
								class="text-sm font-medium text-primary underline-offset-4 hover:underline"
							>
								{t('login.forgotPassword')}
							</a>
						</div>
						<div class="relative">
							<LockKeyholeIcon
								class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
							/>
							<Input
								id="password-{id}"
								type="password"
								autocomplete="current-password"
								class="pl-8"
								disabled={isSubmitting}
								bind:value={password}
							/>
						</div>
					</Field>
					{#if visibleError}
						<p
							class="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive"
						>
							{visibleError}
						</p>
					{/if}
					<Field>
						<Button type="submit" disabled={isSubmitting}>
							{#if isSubmitting}
								<Spinner />
							{/if}
							{isSubmitting ? t('login.submitting') : t('login.submit')}
						</Button>
					</Field>
				</FieldGroup>
			</form>
		</Card.Content>
	</Card.Root>
</div>
