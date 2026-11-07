<script lang="ts">
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	type Props = {
		count: number;
		total: number;
		actionLabel: string;
		onaction?: () => void;
	};

	let { count, total, actionLabel, onaction }: Props = $props();
</script>

{#if count > 0}
	<div
		class="fixed inset-x-0 bottom-0 z-10 border-t border-neutral-200 bg-white/95 px-4 pt-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] backdrop-blur"
	>
		<div class="mx-auto flex max-w-xl items-center justify-between gap-3">
			<div>
				<p class="text-xs text-neutral-500">{t('shop.cartTitle')} ({count})</p>
				<p class="text-base font-bold text-[#333333]">{formatIdr(total)}</p>
			</div>
			<button
				type="button"
				onclick={() => onaction?.()}
				class="flex-1 rounded-full bg-[#ff6534] py-3 text-sm font-bold text-white transition-colors hover:bg-[#e5551f]"
			>
				{actionLabel}
			</button>
		</div>
	</div>
{/if}
