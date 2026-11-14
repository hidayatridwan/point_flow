<script lang="ts">
	import MinusIcon from '@lucide/svelte/icons/minus';
	import PlusIcon from '@lucide/svelte/icons/plus';

	// Penanda arah baris laporan: `add` (+) menambah, `deduct` (−) mengurangi,
	// `none` untuk baris subtotal/informasi tanpa arah. Ikon sengaja disertai
	// teks +/− pada nominal agar tetap terbaca tanpa bergantung pada warna.
	let { effect }: { effect: 'add' | 'deduct' | 'none' } = $props();
</script>

{#if effect === 'add'}
	<span
		class="flex size-5 shrink-0 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
		aria-hidden="true"
	>
		<PlusIcon class="size-3.5" />
	</span>
{:else if effect === 'deduct'}
	<span
		class="flex size-5 shrink-0 items-center justify-center rounded-full bg-rose-500/10 text-rose-600 dark:text-rose-400"
		aria-hidden="true"
	>
		<MinusIcon class="size-3.5" />
	</span>
{:else}
	<span class="size-5 shrink-0" aria-hidden="true"></span>
{/if}
