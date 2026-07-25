<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listRgpdRequests, updateRgpdRequestStatus, type RgpdRequest } from '$lib/api';

  let requests = $state<RgpdRequest[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let copied = $state(false);

  // Local state for status updates (not persisted yet)
  let localStatuses = $state<Record<string, string>>({});

  onMount(async () => {
    try {
      const data = await listRgpdRequests();
      requests = data;
      // Initialize local statuses from fetched data
      localStatuses = data.reduce((acc, r) => {
        acc[r.id] = r.status_id;
        return acc;
      }, {} as Record<string, string>);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? requests[0]?.id ?? null);
  const selected = $derived(requests.find((r) => r.id === selectedId));

  function getTypeLabel(typeId: string): string {
    switch (typeId) {
      case 'type_001': return 'Accès';
      case 'type_002': return 'Rectification';
      case 'type_003': return 'Effacement';
      case 'type_004': return 'Opposition';
      case 'type_005': return 'Déréférencement';
      default: return typeId;
    }
  }

  function getStatusLabel(statusId: string): string {
    switch (statusId) {
      case 'status_001': return 'Brouillon';
      case 'status_002': return 'Prête à envoyer';
      case 'status_003': return 'Envoyée';
      case 'status_004': return 'Répondue';
      default: return statusId;
    }
  }

  function getStatusColor(statusId: string): string {
    switch (statusId) {
      case 'status_002': return 'var(--mantis-warn)';
      case 'status_003': return 'var(--mantis-accent)';
      case 'status_004': return 'var(--mantis-ok)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  async function setStatus(id: string, status: string) {
    try {
      await updateRgpdRequestStatus(id, status);
      localStatuses = { ...localStatuses, [id]: status };
    } catch (e) {
      error = String(e);
    }
  }

  async function copyDraft(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      /* clipboard may be unavailable outside secure context */
    }
  }

  const currentStatus = $derived(selected ? localStatuses[selected.id] ?? selected.status_id : '');
</script>

<section class="wf-view">
  <GuideHeader
    title="DPO / RGPD"
    question="Comment préparer ma demande de droits, sans me tromper ?"
    intro="MANTIS prépare le texte et le suivi. Vous seul envoyez la demande (mail, formulaire ou courrier)."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des demandes...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if requests.length === 0}
    <div class="glass-card"><p class="muted">Aucune demande RGPD enregistrée.</p></div>
  {:else}
    <div class="split-layout">
      <!-- Liste des demandes -->
      <div class="glass-card list-panel">
        <h2>Démarches</h2>
        <ul class="item-list">
          {#each requests as req (req.id)}
            <li>
              <a class="list-item" class:active={selectedId === req.id} href={`/dpo?id=${req.id}`}>
                <div class="item-header">
                  <span class="item-title">{req.target}</span>
                  <span class="badge" style={`color: ${getStatusColor(localStatuses[req.id] ?? req.status_id)}; border-color: ${getStatusColor(localStatuses[req.id] ?? req.status_id)};`}>
                    {getStatusLabel(localStatuses[req.id] ?? req.status_id)}
                  </span>
                </div>
                <p class="item-desc">{getTypeLabel(req.type_id)}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Détail de la demande -->
      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{getTypeLabel(selected.type_id)} — {selected.target}</h3>
          </div>
          
          <p class="summary">
            Étapes : vérifier la cible → copier le brouillon → envoyer vous-même → noter l’envoi ici.
          </p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Organisation cible</dt>
              <dd>{selected.target}</dd>
            </div>
            <div class="field">
              <dt>Contact DPO / privacy</dt>
              <dd>{selected.dpo_contact}</dd>
            </div>
            <div class="field">
              <dt>Données concernées</dt>
              <dd>{selected.data_summary}</dd>
            </div>
          </div>

          <div class="actions-section">
            <h4>Aperçu du brouillon</h4>
            <pre class="draft-preview">{selected.draft_preview}</pre>
          </div>

          <div class="actions-section">
            <h4>Actions</h4>
            <div class="action-buttons">
              <button class="wf-btn" onclick={() => copyDraft(selected.draft_preview)}>
                {copied ? 'Copié !' : 'Copier le brouillon'}
              </button>
              
              {#if currentStatus === 'status_001'}
                <button class="wf-btn primary" onclick={() => setStatus(selected.id, 'status_002')}>
                  Marquer prête à envoyer
                </button>
              {:else if currentStatus === 'status_002'}
                <button class="wf-btn primary" onclick={() => setStatus(selected.id, 'status_003')}>
                  J'ai envoyé
                </button>
              {:else if currentStatus === 'status_003'}
                <button class="wf-btn primary" onclick={() => setStatus(selected.id, 'status_004')}>
                  Marquer comme répondue
                </button>
              {/if}
            </div>
            <p class="muted" style="margin-top: 0.75rem;">
              Aucun envoi automatique. Aucun secret stocké. Le changement de statut est persisté localement.
            </p>
          </div>
        {:else}
          <p class="muted">Sélectionnez une démarche.</p>
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

  .draft-preview {
    margin: 0;
    padding: 1rem;
    border: 1px solid var(--mantis-border);
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.3);
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.8rem;
    white-space: pre-wrap;
    line-height: 1.5;
    max-height: 300px;
    overflow-y: auto;
  }

  .action-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
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
