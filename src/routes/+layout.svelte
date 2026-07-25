<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import Titlebar from '$lib/Titlebar.svelte';
	import { page } from '$app/stores';

	let { children } = $props();

	const navItems = [
		{ label: 'Centre de posture', path: '/posture' },
		{ label: 'Dossiers', path: '/dossiers' },
		{ label: 'Graphe', path: '/graphe' },
		{ label: 'Identités', path: '/identites' },
		{ label: 'Veille', path: '/veille' },
		{ label: 'Expositions', path: '/expositions' },
		{ label: 'Incidents', path: '/incidents' },
		{ label: 'Actions', path: '/actions' },
		{ label: 'DPO', path: '/dpo' },
		{ label: 'Guides', path: '/guides' },
		{ label: 'Rapports', path: '/rapports' },
		{ label: 'Paramètres', path: '/parametres' }
	];

	const currentPath = $derived($page.url.pathname);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>MANTIS POSTURE</title>
</svelte:head>

<div class="app">
	<Titlebar />
	<div class="shell">
	<aside class="sidebar">
		<div class="sidebar-brand">
			<span class="brand-mark" aria-hidden="true">M</span>
			<div class="brand-text">
				<span class="brand-name">MANTIS</span>
				<span class="brand-sub">POSTURE</span>
			</div>
		</div>

		<nav class="sidebar-nav" aria-label="Sections">
			<ul>
				{#each navItems as item (item.path)}
					<li>
						<a href={item.path} class="nav-item" class:active={currentPath === item.path}>
							{item.label}
						</a>
					</li>
				{/each}
			</ul>
		</nav>
	</aside>

		<main class="content">
			{@render children()}
		</main>
	</div>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
		background: var(--mantis-bg);
	}

	.shell {
		display: flex;
		flex: 1;
		min-height: 0;
	}

	.sidebar {
		width: 240px;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		padding: 1.25rem 0.75rem;
		background: var(--mantis-bg-sidebar);
		border-right: 1px solid var(--mantis-border);
		overflow-y: auto;
	}

	.sidebar-brand {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0 0.5rem;
		margin-bottom: 0.5rem;
	}

	.brand-mark {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border: 1px solid var(--mantis-border-strong);
		border-radius: 8px;
		background: var(--mantis-bg-solid);
		color: var(--mantis-accent);
		font-weight: 700;
		font-size: 1.1rem;
	}

	.brand-text {
		display: flex;
		flex-direction: column;
		line-height: 1.1;
	}

	.brand-name {
		font-weight: 700;
		letter-spacing: 0.08em;
		font-size: 1rem;
	}

	.brand-sub {
		font-size: 0.7rem;
		letter-spacing: 0.3em;
		color: var(--mantis-text-muted);
	}

	.sidebar-nav ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.nav-item {
		display: block;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		border-left: 2px solid transparent;
		font-size: 0.85rem;
		color: var(--mantis-text-muted);
		text-decoration: none;
		transition: all 0.15s ease;
	}

	.nav-item:hover {
		background: rgba(255, 255, 255, 0.03);
		color: var(--mantis-text);
	}

	.nav-item.active {
		background: rgba(91, 141, 239, 0.1);
		border-left-color: var(--mantis-accent);
		color: var(--mantis-text);
		font-weight: 600;
	}

	.content {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		background: linear-gradient(180deg, var(--mantis-bg) 0%, #0a0b0e 100%);
	}
</style>
