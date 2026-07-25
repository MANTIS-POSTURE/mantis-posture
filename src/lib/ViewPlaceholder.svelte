<script lang="ts">
	interface Props {
		title: string;
		description?: string;
	}

	let { title, description }: Props = $props();
</script>

<section class="view">
	<header class="view-header">
		<h1>{title}</h1>
		{#if description}
			<p class="view-sub">{description}</p>
		{/if}
	</header>
	<div class="placeholder-card">
		<p class="placeholder-title">Vue à venir</p>
		<p class="placeholder-text">
			Cette section sera développée dans les prochaines phases de la roadmap.
		</p>
	</div>
</section>

<style>
	.view {
		max-width: 900px;
		margin: 0 auto;
		padding: 2.5rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.view-header h1 {
		margin: 0;
		font-size: 1.5rem;
		font-weight: 700;
	}

	.view-sub {
		margin: 0.25rem 0 0;
		color: var(--mantis-text-muted);
	}

	.placeholder-card {
		background: var(--mantis-bg-raised);
		border: 1px solid var(--mantis-border);
		border-radius: 12px;
		padding: 3rem 2rem;
		text-align: center;
	}

	.placeholder-title {
		margin: 0 0 0.5rem;
		font-size: 1.1rem;
		font-weight: 600;
		color: var(--mantis-text-muted);
	}

	.placeholder-text {
		margin: 0;
		font-size: 0.9rem;
		color: var(--mantis-text-muted);
	}
</style>