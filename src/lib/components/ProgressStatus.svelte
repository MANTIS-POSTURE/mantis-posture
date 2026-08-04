<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props { value: number; max: number; label: string; detail?: string; tone?: 'accent'|'ai'; }
	let { value, max, label, detail, tone='accent' }: Props = $props();
	let percent = $derived(max > 0 ? Math.min(100, Math.max(0, value / max * 100)) : 0);
</script>

<div class="progress-status tone-{tone}" role="status" aria-label={`${t(label)} : ${Math.round(percent)} %`}>
	<div class="progress-head"><strong>{t(label)}</strong><span>{Math.round(percent)} %</span></div>
	<div class="track"><span style={`width:${percent}%`}></span></div>
	{#if detail}<small>{t(detail)}</small>{/if}
</div>

<style>
	.progress-status { --progress:var(--ui-accent); display:grid; gap:.45rem; padding:.8rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-md); background:var(--ui-material-solid); }.tone-ai { --progress:var(--ui-ai); }
	.progress-head { display:flex; justify-content:space-between; gap:1rem; font-size:.78rem; }.progress-head span,small { color:var(--ui-text-secondary); }.track { height:7px; overflow:hidden; border-radius:var(--radius-pill); background:var(--ui-surface-3); box-shadow:inset 0 1px 2px rgba(0,0,0,.4); }.track span { display:block; height:100%; border-radius:inherit; background:var(--progress); transition:width var(--duration-slow) var(--ease-emphasized); } small { font-size:.72rem; }
</style>
