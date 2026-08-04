<script lang="ts">
	let { current }: { current: 'exposition' | 'incident' | 'action' | 'rgpd' } = $props();

	const steps = [
		{ id: 'exposition', index: '01', label: 'Constater', description: 'Vérifier une exposition', href: '/expositions' },
		{ id: 'incident', index: '02', label: 'Décider', description: 'Qualifier ce qui compte', href: '/incidents' },
		{ id: 'action', index: '03', label: 'Agir', description: 'Suivre une mesure concrète', href: '/actions' },
		{ id: 'rgpd', index: '04', label: 'Demander', description: 'Préparer une démarche RGPD', href: '/dpo' }
	] as const;
	const currentIndex = $derived(steps.findIndex((step) => step.id === current));
</script>

<nav class="remediation-journey" aria-label="Parcours de remédiation">
	{#each steps as step, index}
		<a href={step.href} class:active={step.id === current} class:complete={index < currentIndex} aria-current={step.id === current ? 'step' : undefined}>
			<span>{index < currentIndex ? '✓' : step.index}</span>
			<strong>{step.label}</strong>
			<small>{step.description}</small>
		</a>
	{/each}
</nav>

<style>
	.remediation-journey { display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); overflow:hidden; border:1px solid var(--ui-border-default); border-radius:var(--radius-md); background:var(--ui-border-default); gap:1px; }
	a { display:grid; grid-template-columns:auto 1fr; gap:.12rem .65rem; min-height:62px; padding:.72rem .82rem; color:var(--ui-text-secondary); background:var(--ui-surface-1); text-decoration:none; transition:background var(--duration-fast),color var(--duration-fast); }
	a:hover { color:var(--ui-text-primary); background:var(--ui-surface-2); }
	a.active { color:var(--ui-text-primary); background:color-mix(in srgb,var(--ui-accent) 7%,var(--ui-surface-1)); box-shadow:inset 0 3px 0 var(--ui-accent); }
	a.complete span { color:var(--ui-success); }
	span { grid-row:1/3; align-self:center; color:var(--ui-text-tertiary); font-family:var(--font-meta); font-size:.68rem; }
	strong { align-self:end; font-size:.78rem; }
	small { color:var(--ui-text-tertiary); font-size:.66rem; line-height:1.25; }
	@media(max-width:820px) { .remediation-journey { grid-template-columns:repeat(2,minmax(0,1fr)); } }
	@media(max-width:520px) { .remediation-journey { grid-template-columns:1fr; } }
</style>
