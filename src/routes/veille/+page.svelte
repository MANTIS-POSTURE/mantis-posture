<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import NextStepBar from '$lib/NextStepBar.svelte';
  import { listOsintModules, runOsintModule, type OsintModule } from '$lib/api';

  let modules = $state<OsintModule[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let running = $state(false);
  let runResult = $state<string | null>(null);

  onMount(async () => {
    try {
      modules = await listOsintModules();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? modules[0]?.id ?? null);
  const selected = $derived(modules.find((m) => m.id === selectedId));

  function getStatusLabel(status: string): string {
    switch (status) {
      case 'planifie': return 'Planifié';
      case 'actif': return 'Actif';
      case 'erreur': return 'Erreur';
      case 'desactive': return 'Désactivé';
      default: return status;
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'actif': return 'var(--mantis-ok)';
      case 'planifie': return 'var(--mantis-warn)';
      case 'erreur': return 'var(--mantis-danger)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  async function executeModule() {
    if (!selected) return;
    running = true;
    runResult = null;
    error = null;
    try {
      const result = await runOsintModule(selected.id);
      runResult = result;
      // Refresh modules to update last_run
      modules = await listOsintModules();
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Veille"
    question="Qu’est-ce qui tourne en arrière-plan pour me surveiller ?"
    intro="Ici vous voyez les routines OSINT configurées. En Phase 1/2, elles sont planifiées mais n'émettent aucune requête réseau réelle."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des routines...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if modules.length === 0}
    <div class="glass-card"><p class="muted">Aucun module de veille configuré.</p></div>
  {:else}
    <div class="split-layout">
      <div class="glass-card list-panel">
        <h2>Routines OSINT</h2>
        <ul class="item-list">
          {#each modules as mod (mod.id)}
            <li>
              <a class="list-item" class:active={selectedId === mod.id} href={`/veille?id=${mod.id}`}>
                <div class="item-header">
                  <span class="item-title">{mod.name}</span>
                  <span class="badge" style={`color: ${getStatusColor(mod.status)}; border-color: ${getStatusColor(mod.status)};`}>
                    {getStatusLabel(mod.status)}
                  </span>
                </div>
                <p class="item-desc">{mod.frequency} · Cible: {mod.target_kind}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.name}</h3>
          </div>
          
          <p class="summary">{selected.description}</p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Cible surveillée</dt>
              <dd>{selected.target_kind}</dd>
            </div>
            <div class="field">
              <dt>Fréquence</dt>
              <dd>{selected.frequency}</dd>
            </div>
            <div class="field">
              <dt>Dernière exécution</dt>
              <dd>{selected.last_run ?? 'Jamais'}</dd>
            </div>
            <div class="field">
              <dt>Prochaine exécution</dt>
              <dd>{selected.next_run ?? 'Non planifiée'}</dd>
            </div>
          </div>

          <div class="actions-section">
            <h4>Exécution manuelle</h4>
            <div class="action-buttons">
              <button class="wf-btn primary" onclick={executeModule} disabled={running}>
                {running ? 'Exécution en cours...' : 'Lancer la routine maintenant'}
              </button>
            </div>
            {#if runResult}
              <div class="run-result">
                <p class="run-title">Résultat de l'exécution :</p>
                <pre>{runResult}</pre>
              </div>
            {/if}
            <p class="muted" style="margin-top: 0.75rem;">
              Note : En Phase 4, cette action interrogera de vraies APIs OSINT de manière sécurisée.
            </p>
          </div>

          <NextStepBar
            hint="La veille réelle arrivera avec l’OSINT contrôlé (Phase 4). En attendant, traitez les priorités du Centre."
            primaryHref="/posture"
            primaryLabel="Voir le centre de posture"
          >
            <a class="wf-btn" href="/identites">Vérifier mes identités</a>
          </NextStepBar>
        {:else}
          <p class="muted">Sélectionnez une routine.</p>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .split-layout {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 1.25rem;
  }

  @media (max-width: 900px) {
    .split-layout {
      grid-template-columns: 1fr;
    }
  }

  .list-panel h2 {
    margin: 0 0 1rem;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .item-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.75rem;
    border: 1px solid var(--mantis-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    text-decoration: none;
    color: inherit;
    transition: border-color 0.12s;
  }

  .list-item:hover {
    border-color: var(--mantis-accent);
  }

  .list-item.active {
    border-color: var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 10%, transparent);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }

  .item-title {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .item-desc {
    margin: 0;
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
  }

  .detail-header {
    margin-bottom: 1rem;
  }

  .detail-header h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .summary {
    margin: 0 0 1.25rem;
    padding: 0.75rem 1rem;
    border-left: 3px solid var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 5%, transparent);
    border-radius: 0 6px 6px 0;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 700px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }

  .field {
    margin: 0;
  }

  .field dt {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mantis-text-muted);
    margin-bottom: 0.3rem;
  }

  .field dd {
    margin: 0;
    font-size: 0.88rem;
    color: var(--mantis-text);
  }

  .actions-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .action-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .run-result {
    margin-top: 1rem;
    padding: 0.75rem 1rem;
    border: 1px solid var(--mantis-border);
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.3);
  }

  .run-title {
    margin: 0 0 0.5rem;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--mantis-text);
  }

  .run-result pre {
    margin: 0;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.8rem;
    white-space: pre-wrap;
    line-height: 1.5;
    color: var(--mantis-text-muted);
  }

  .badge {
    display: inline-block;
    padding: 0.15rem 0.45rem;
    border: 1px solid;
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    align-self: flex-start;
  }
</style>
