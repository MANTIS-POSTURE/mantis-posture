<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props {
		title: string;
		/** Main question this view answers for the user */
		question: string;
		/** Optional short plain-language intro */
		intro?: string;
	}

	let { title, question, intro }: Props = $props();
</script>

<header class="guide-header">
	<p class="guide-kicker">{t('Espace personnel')}</p>
	<div class="guide-heading"><h1>{t(title)}</h1><span aria-hidden="true"></span></div>
	<p class="guide-question">{t(question)}</p>
	{#if intro}
		<p class="guide-intro">{t(intro)}</p>
	{/if}
</header>

<style>
	.guide-header h1 {
		margin: 0;
		font-size: clamp(1.55rem, 2.2vw, 2rem);
		font-weight: 680;
		letter-spacing: -.035em;
	}
	.guide-header { display:grid; gap:.3rem; padding-bottom:.15rem; }
	.guide-kicker { margin:0; color:var(--ui-accent); font-family:var(--font-meta); font-size:.67rem; font-weight:650; letter-spacing:.13em; text-transform:uppercase; }
	.guide-heading { display:flex; align-items:center; gap:.75rem; }.guide-heading span { width:28px; height:1px; background:var(--ui-accent); opacity:.65; }

	.guide-question {
		margin: 0.1rem 0 0;
		font-size: 0.98rem;
		font-weight: 600;
		color: var(--mantis-text);
	}

	.guide-intro {
		margin: 0.45rem 0 0;
		font-size: 0.9rem;
		color: var(--mantis-text-muted);
		max-width: 52rem;
		line-height: 1.5;
	}
</style>
