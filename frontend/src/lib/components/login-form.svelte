<script lang="ts">
	import loginImage from '$lib/assets/place_holder.jpg';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { FieldGroup, FieldDescription } from '$lib/components/ui/field/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { cn } from '$lib/utils.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import type { HTMLAttributes } from 'svelte/elements';
	import { z } from 'zod';

	const loginFormSchema = z.object({
		email: z.string().trim().min(1, 'Email wajib diisi.').email('Format email tidak valid.'),
		password: z.string().min(1, 'Password wajib diisi.')
	});

	type LoginCredentials = z.infer<typeof loginFormSchema>;

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

	const form = superForm(
		defaults(
			{
				email: '',
				password: ''
			},
			zod4(loginFormSchema)
		),
		{
			SPA: true,
			validationMethod: 'submit-only',
			validators: zod4Client(loginFormSchema)
		}
	);
	const { form: formData, validateForm } = form;

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) {
			return;
		}

		await onSubmit?.({
			email: result.data.email,
			password: result.data.password
		});
	}
</script>

<div class={cn('flex flex-col gap-6', className)} {...restProps}>
	<Card.Root class="overflow-hidden p-0">
		<Card.Content class="grid p-0 md:grid-cols-2">
			<form class="p-6 md:p-8" onsubmit={handleSubmit}>
				<FieldGroup>
					<div class="flex flex-col items-center gap-2 text-center">
						<h1 class="text-2xl font-bold">Masuk ke POS</h1>
						<p class="text-balance text-muted-foreground">Gunakan akun admin atau kasir.</p>
					</div>
					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Email</Form.Label>
								<Input
									{...props}
									type="email"
									placeholder="admin@example.com"
									autocomplete="email"
									disabled={isSubmitting}
									bind:value={$formData.email}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					<Form.Field {form} name="password">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Password</Form.Label>
								<Input
									{...props}
									type="password"
									autocomplete="current-password"
									disabled={isSubmitting}
									bind:value={$formData.password}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
					{#if errorMessage}
						<p
							class="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive"
						>
							{errorMessage}
						</p>
					{/if}
					<Form.Button disabled={isSubmitting}>
						{isSubmitting ? 'Processing...' : 'Login'}
					</Form.Button>
				</FieldGroup>
			</form>
			<div class="relative hidden bg-muted md:block">
				<img
					src={loginImage}
					alt="POS terminal"
					class="absolute inset-0 h-full w-full object-cover dark:brightness-[0.2] dark:grayscale"
				/>
			</div>
		</Card.Content>
	</Card.Root>
	<FieldDescription class="px-6 text-center">
		Akun Anda tetap terlindungi selama menggunakan aplikasi.
	</FieldDescription>
</div>
