<script lang="ts">
	import { localeState, setLocale, t } from '$lib/i18n/locale.svelte.js';
	import type { Locale } from '$lib/i18n/messages.js';

	const options: { value: Locale; label: string }[] = [
		{ value: 'id', label: 'ID' },
		{ value: 'en', label: 'EN' }
	];
</script>

<div
	class="inline-flex items-center gap-1 rounded-full bg-white p-1 shadow-sm"
	role="group"
	aria-label={t('language.label')}
>
	{#each options as option (option.value)}
		<button
			type="button"
			onclick={() => setLocale(option.value)}
			aria-pressed={localeState.locale === option.value}
			class="rounded-full px-2.5 py-1 text-xs font-bold transition-colors"
			class:bg-[#ff6534]={localeState.locale === option.value}
			class:text-white={localeState.locale === option.value}
			class:text-neutral-500={localeState.locale !== option.value}
		>
			{option.label}
		</button>
	{/each}
</div>
