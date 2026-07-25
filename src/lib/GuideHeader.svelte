<script lang="ts">
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
	<h1>{title}</h1>
	<p class="guide-question">{question}</p>
	{#if intro}
		<p class="guide-intro">{intro}</p>
	{/if}
</header>

<style>
	.guide-header h1 {
		margin: 0;
		font-size: 1.5rem;
		font-weight: 700;
	}

	.guide-question {
		margin: 0.4rem 0 0;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--mantis-text);
	}

	.guide-intro {
		margin: 0.45rem 0 0;
		font-size: 0.88rem;
		color: var(--mantis-text-muted);
		max-width: 52rem;
		line-height: 1.5;
	}
</style>
