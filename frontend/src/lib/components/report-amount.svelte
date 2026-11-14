<script lang="ts">
	import { formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';

	// Nominal baris laporan. `add`/`deduct` memberi awalan +/− sekaligus warna
	// hijau/merah supaya arahnya jelas bahkan saat dicetak hitam-putih.
	let {
		value,
		effect = 'none',
		class: className
	}: {
		value: number;
		effect?: 'add' | 'deduct' | 'none';
		class?: string;
	} = $props();
</script>

<span
	class={cn(
		'tabular-nums',
		effect === 'add' && 'text-emerald-600 dark:text-emerald-400',
		effect === 'deduct' && 'text-rose-600 dark:text-rose-400',
		className
	)}
>
	{#if effect === 'add'}+{:else if effect === 'deduct'}−{/if}{formatIdr(value)}
</span>
