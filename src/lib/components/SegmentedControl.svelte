<script lang="ts">
	import { t } from '$lib/i18n';
	type Option = { value: string; label: string };

	let { options, value, label, onChange }: { options: Option[]; value: string; label: string; onChange: (value: string) => void } = $props();
</script>

<div class="segmented" role="group" aria-label={t(label)}>
	{#each options as option (option.value)}
		<button type="button" class:active={value === option.value} aria-pressed={value === option.value} onclick={() => onChange(option.value)}>{t(option.label)}</button>
	{/each}
</div>

<style>
	.segmented { display:inline-flex; align-items:center; gap:2px; padding:3px; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-md); background:var(--ui-material-solid); }
	button { min-height:34px; padding:.4rem .72rem; border:1px solid transparent; border-radius:var(--radius-sm); color:var(--ui-text-secondary); background:transparent; font-size:.8rem; font-weight:600; cursor:pointer; transition:color var(--duration-fast),background var(--duration-fast),border-color var(--duration-fast); }
	button:hover { color:var(--ui-text-primary); background:var(--ui-surface-2); }
	button.active { border-color:color-mix(in srgb,var(--ui-accent) 48%,var(--ui-border-default)); color:var(--ui-text-inverse); background:var(--ui-accent); box-shadow:inset 0 1px 0 var(--ui-rim-light); }
</style>
