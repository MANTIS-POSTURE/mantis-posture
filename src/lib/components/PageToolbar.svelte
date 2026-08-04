<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '$lib/i18n';

	let { label = 'Outils de la page', children }: { label?: string; children: Snippet } = $props();
</script>

	<div class="page-toolbar" role="group" aria-label={t(label)}>
	{@render children()}
</div>

<style>
	.page-toolbar { display:flex; flex-wrap:wrap; align-items:center; justify-content:space-between; gap:.65rem 1rem; min-height:52px; padding:.55rem .65rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-md); background:var(--ui-material-solid); box-shadow:inset 0 1px 0 var(--ui-rim-light); }
	.page-toolbar :global(label) { display:flex; align-items:center; gap:.4rem; color:var(--ui-text-secondary); font-size:.8rem; }
	.page-toolbar :global(select) { min-height:34px; margin:0; padding:.35rem 1.8rem .35rem .55rem; }
</style>
