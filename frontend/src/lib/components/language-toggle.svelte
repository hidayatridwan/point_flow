<script lang="ts">
	import { localeLabels } from '$lib/i18n/messages.js';
	import { localeState, t, toggleLocale } from '$lib/i18n/locale.svelte.js';
	import { Button } from '$lib/components/ui/button/index.js';

	const label = $derived(
		localeState.locale === 'id' ? t('language.switchToEnglish') : t('language.switchToIndonesian')
	);
</script>

<Button
	type="button"
	variant="ghost"
	size="icon-sm"
	class="text-xs font-semibold"
	aria-label={label}
	onclick={toggleLocale}
>
	{localeLabels[localeState.locale]}
	<span class="sr-only">{label}</span>
</Button>
