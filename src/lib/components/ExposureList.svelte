<script lang="ts">
  import { onMount } from 'svelte';
  import { listExposures, type Exposure } from '$lib/api';
	import StatePanel from './StatePanel.svelte';
  import { t } from '$lib/i18n';

  let exposures = $state<Exposure[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  const visibleExposures = $derived(exposures.slice(0, 3));

  onMount(async () => {
    try {
      exposures = await listExposures();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString(document.documentElement.lang === 'en' ? 'en-US' : 'fr-FR', { year: 'numeric', month: 'long', day: 'numeric' });
  }

  function getSeverityColor(sev: string): string {
    switch (sev) {
      case 'critique': return 'var(--mantis-danger)';
      case 'élevée': return 'var(--ui-danger)';
      case 'modérée': return 'var(--mantis-warn)';
      default: return 'var(--mantis-text-muted)';
    }
  }
</script>

<div class="glass-card">
  <div class="list-heading"><div><p class="eyebrow">{t('Observations retenues')}</p><h2>{t('Expositions')}</h2></div>{#if exposures.length}<span>{exposures.length}</span>{/if}</div>
  
  {#if loading}
    <StatePanel compact tone="info" title={t('Chargement des expositions')} />
  {:else if error}
    <StatePanel compact tone="danger" title={t('Expositions indisponibles')} message={error} />
  {:else if exposures.length === 0}
    <StatePanel compact tone="success" title={t('Aucune exposition enregistrée')} />
  {:else}
    <div class="list">
      {#each visibleExposures as exposure (exposure.id)}
        <a class="item" href={`/expositions?id=${exposure.id}`}>
          <div class="item-header">
            <div>
              <h3>{t(exposure.title)}</h3>
              <p class="date">{t('Détecté le')} {formatDate(exposure.discovered_at)}</p>
            </div>
            <span class="badge" style={`color: ${getSeverityColor(exposure.severity)}; border-color: ${getSeverityColor(exposure.severity)};`}>
              {exposure.severity}
            </span>
          </div>
          <p class="item-desc">{exposure.what}</p>
        </a>
      {/each}
    </div>
	<a class="list-footer" href="/expositions">{t('Voir toutes les expositions →')}</a>
  {/if}
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
	.list-heading { display:flex; align-items:start; justify-content:space-between; gap:1rem; margin-bottom:.85rem; }.list-heading h2 { margin:.2rem 0 0; }.list-heading .eyebrow { margin:0; color:var(--ui-warning); text-transform:uppercase; }.list-heading>span { display:grid; place-items:center; min-width:28px; height:28px; border:1px solid var(--ui-border-default); border-radius:var(--radius-pill); color:var(--ui-text-secondary); font-family:var(--font-meta); font-size:.72rem; }.list-footer { display:inline-block; margin-top:.85rem; color:var(--ui-link); font-size:.78rem; font-weight:620; text-decoration:none; }

  .item {
    display: block;
    padding: 1rem;
    border: 1px solid var(--mantis-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    text-decoration: none;
    color: inherit;
    transition: border-color 0.12s;
  }

  .item:hover {
    border-color: var(--mantis-accent);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.5rem;
  }

  .item-header h3 {
    margin: 0 0 0.15rem;
    font-size: 1rem;
    font-weight: 600;
  }

  .date {
    margin: 0;
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
  }

  .item-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--mantis-text-muted);
  }

  .badge {
    padding: 0.15rem 0.45rem;
    border: 1px solid;
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
  }
</style>
