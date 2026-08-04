<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import RemediationJourney from '$lib/components/RemediationJourney.svelte';
  import { listActions, updateActionTracking, listActionEvents, listActionEvidence, addActionEvidence, type Action, type ActionEvent, type ActionEvidence } from '$lib/api';
  import { activeIdentityId } from '$lib/active-identity';
  import { recommendGuideForContext } from '$lib/guides';

  let actions = $state<Action[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let events = $state<ActionEvent[]>([]);
  let evidence = $state<ActionEvidence[]>([]);
  let trackingNote = $state('');
  let actor = $state<'moi' | 'site' | ''>('');
  let blockedReason = $state('');
  let evidenceKind = $state<ActionEvidence['kind']>('url');
  let evidenceLocator = $state('');
  let evidenceDescription = $state('');

  async function refresh() {
    loading = true;
    try {
      actions = await listActions($activeIdentityId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  const selectedId = $derived($page.url.searchParams.get('id') ?? actions[0]?.id ?? null);
  const selected = $derived(actions.find((a) => a.id === selectedId));
  const preventiveGuide = $derived(selected ? recommendGuideForContext(`${selected.title} ${selected.guidance}`) : null);

  $effect(() => {
    const action = selected;
    if (!action) return;
    trackingNote = action.tracking_note ?? '';
    actor = action.actor ?? '';
    blockedReason = action.blocked_reason ?? '';
    Promise.all([listActionEvents(action.id), listActionEvidence(action.id)])
      .then(([nextEvents, nextEvidence]) => { events = nextEvents; evidence = nextEvidence; })
      .catch((e) => { error = String(e); });
  });
  onMount(() => activeIdentityId.subscribe(() => { void refresh(); }));

  function getPriorityLabel(id: string): string {
    switch (id) {
      case 'prio_001': return 'Basse';
      case 'prio_002': return 'Moyenne';
      case 'prio_003': return 'Haute';
      case 'prio_004': return 'Critique';
      default: return id;
    }
  }

  function getDifficultyLabel(id: string): string {
    switch (id) {
      case 'diff_001': return 'Facile';
      case 'diff_002': return 'Moyenne';
      case 'diff_003': return 'Difficile';
      default: return id;
    }
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case 'a_faire': return 'À faire';
      case 'en_cours': return 'En cours';
      case 'effectue_moi': return 'Effectuée par moi';
      case 'effectue_site': return 'Effectuée par le site';
      case 'en_attente': return 'En attente';
      case 'impossible': return 'Impossible';
      case 'ignore': return 'Ignorée';
      case 'faite': return 'Faite';
      default: return status;
    }
  }

  function getPriorityColor(id: string): string {
    switch (id) {
      case 'prio_004': return 'var(--mantis-danger)';
      case 'prio_003': return 'var(--ui-warning)';
      case 'prio_002': return 'var(--mantis-warn)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'effectue_moi': case 'effectue_site': case 'faite': return 'var(--mantis-ok)';
      case 'en_cours': return 'var(--mantis-accent)';
      case 'impossible': return 'var(--mantis-danger)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function parseGuidance(guidance: string): string[] {
    try {
      return JSON.parse(guidance);
    } catch {
      return [];
    }
  }

  async function updateStatus(newStatus: string) {
    if (!selected) return;
    try {
      await updateActionTracking(selected.id, newStatus, trackingNote || null, actor || null, blockedReason || null);
      // Update local state to reflect change immediately
      actions = actions.map(a => a.id === selected.id ? { ...a, status: newStatus } : a);
      events = await listActionEvents(selected.id);
    } catch (e) {
      error = String(e);
    }
  }

  async function saveTracking() {
    if (!selected) return;
    await updateActionTracking(selected.id, selected.status, trackingNote || null, actor || null, blockedReason || null);
    actions = actions.map(a => a.id === selected.id ? { ...a, tracking_note: trackingNote || null, actor: actor || null, blocked_reason: blockedReason || null } : a);
    events = await listActionEvents(selected.id);
  }

  async function saveEvidence() {
    if (!selected || !evidenceLocator.trim()) return;
    try {
      await addActionEvidence(selected.id, evidenceKind, evidenceLocator.trim(), evidenceDescription.trim() || null);
      evidenceLocator = ''; evidenceDescription = '';
      evidence = await listActionEvidence(selected.id);
    } catch (e) { error = String(e); }
  }
</script>

<section class="wf-view actions-view">
  <GuideHeader
    title="Mes prochaines actions"
    question="Que puis-je faire maintenant pour améliorer ma situation ?"
    intro="Chaque action vous guide pas à pas. Notez votre avancée et, si utile, gardez une preuve. Ne saisissez jamais de mot de passe ici."
  />
  <RemediationJourney current="action" />

  {#if loading}
    <StatePanel tone="info" title="Préparation du plan d’action" message="Lecture des actions et de leur suivi local…" />
  {:else if error}
    <StatePanel tone="danger" title="Plan d’action indisponible" message={error} />
  {:else if actions.length === 0}
    <StatePanel tone="success" title="Aucune action en attente" message="Les mesures créées depuis une exposition ou un point à traiter apparaîtront ici." />
  {:else}
    <div class="split-layout">
      <!-- Liste des actions -->
      <div class="glass-card list-panel">
        <div class="list-heading"><div><p class="eyebrow">Agir</p><h2>À faire et en cours</h2></div><a href="/actions/new" aria-label="Créer une action">＋</a></div>
        <ul class="item-list">
          {#each actions as act (act.id)}
            <li>
              <a class="list-item" class:active={selectedId === act.id} href={`/actions?id=${act.id}`}>
                <div class="item-header">
                  <span class="item-title">{act.title}</span>
                  <span class="badge" style={`color: ${getPriorityColor(act.priority_id)}; border-color: ${getPriorityColor(act.priority_id)};`}>
                    {getPriorityLabel(act.priority_id)}
                  </span>
                </div>
                <p class="item-desc">
                  {getDifficultyLabel(act.difficulty_id)} · avant le {new Date(act.deadline).toLocaleDateString('fr-FR')}
                </p>
                <span class="status-pill" style={`color: ${getStatusColor(act.status)}; border-color: ${getStatusColor(act.status)};`}>
                  {getStatusLabel(act.status)}
                </span>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Détail de l'action -->
      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.title}</h3>
            <div class="badges">
              <span class="badge" style={`color: ${getPriorityColor(selected.priority_id)}; border-color: ${getPriorityColor(selected.priority_id)};`}>
                {getPriorityLabel(selected.priority_id)}
              </span>
              <span class="badge" style={`color: ${getStatusColor(selected.status)}; border-color: ${getStatusColor(selected.status)};`}>
                {getStatusLabel(selected.status)}
              </span>
            </div>
          </div>
          
          <div class="decision-callout"><span>Objectif vérifiable</span><strong>{selected.proof_expected}</strong><small>Difficulté {getDifficultyLabel(selected.difficulty_id)} · échéance {new Date(selected.deadline).toLocaleDateString('fr-FR')}</small></div>

          {#if preventiveGuide}
            <a class="preventive-guide" href={`/guides?id=${preventiveGuide.id}`} style={`--guide-accent:${preventiveGuide.accent}`}>
              <img src={preventiveGuide.image} alt="" />
              <span><small>Éviter que cela se reproduise</small><strong>{preventiveGuide.shortTitle}</strong><p>{preventiveGuide.summary}</p></span>
              <b>Lire le guide →</b>
            </a>
          {/if}
          
          <div class="detail-grid">
            <div class="field">
              <dt>Comment faire (étapes)</dt>
              <dd>
                <ol class="steps-list">
                  {#each parseGuidance(selected.guidance) as step, i (i)}
                    <li>{step}</li>
                  {/each}
                </ol>
              </dd>
            </div>
          </div>

          <div class="actions-section">
            <h4>Statut de l'action</h4>
            <div class="tracking-grid">
              <label>État<select value={selected.status} onchange={(e) => updateStatus((e.currentTarget as HTMLSelectElement).value)}>
                <option value="a_faire">À faire</option><option value="en_cours">En cours</option><option value="effectue_moi">Effectuée par moi</option><option value="effectue_site">Effectuée par le site</option><option value="en_attente">En attente</option><option value="impossible">Impossible</option><option value="ignore">Ignorée</option>
              </select></label>
              <label>Acteur<select bind:value={actor}><option value="">Non précisé</option><option value="moi">Moi</option><option value="site">Site</option></select></label>
            </div>
            <label>Note de suivi<textarea bind:value={trackingNote} rows="3" placeholder="Ce qui a été vérifié, décidé ou bloqué…"></textarea></label>
            <label>Motif si bloquée<textarea bind:value={blockedReason} rows="2" placeholder="Facultatif"></textarea></label>
            <button class="wf-btn primary" onclick={saveTracking}>Enregistrer le suivi</button>
          </div>

          <div class="actions-section">
            <h4>Preuves de réalisation</h4>
            {#if evidence.length}<ul class="evidence-list">{#each evidence as item}<li><strong>{item.kind}</strong> · {item.locator}{#if item.description}<small>{item.description}</small>{/if}</li>{/each}</ul>{:else}<p class="muted">Aucune preuve enregistrée.</p>{/if}
            <div class="tracking-grid"><label>Type<select bind:value={evidenceKind}><option value="url">URL</option><option value="fichier">Fichier local</option><option value="note">Note</option><option value="hash">Hash</option></select></label><label>Référence<input bind:value={evidenceLocator} maxlength="2048" placeholder="URL, chemin contrôlé ou hash" /></label></div>
            <label>Description<textarea bind:value={evidenceDescription} rows="2" placeholder="Ce que cette preuve démontre"></textarea></label>
            <button class="wf-btn" onclick={saveEvidence} disabled={!evidenceLocator.trim()}>Ajouter la preuve</button>
          </div>

          <details class="secondary-section">
            <summary>Historique de l’action <span>{events.length}</span></summary>
            {#if events.length}<ul class="evidence-list">{#each events as event}<li><strong>{getStatusLabel(event.to_status)}</strong> · {new Date(event.created_at).toLocaleString('fr-FR')}{#if event.note}<small>{event.note}</small>{/if}</li>{/each}</ul>{:else}<p class="muted">Aucun changement enregistré.</p>{/if}
          </details>

          {#if selected.incident_id}
            <div class="actions-section">
              <h4>Contexte</h4>
              <p class="muted">Cette action est liée à un incident spécifique.</p>
              <a href={`/incidents?id=${selected.incident_id}`} class="wf-btn" style="margin-top: 0.75rem;">
                Voir l'incident lié
              </a>
            </div>
          {/if}
        {:else}
          <StatePanel compact title="Sélectionnez une action" message="Ses étapes, sa preuve attendue et son suivi apparaîtront ici." />
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .tracking-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .75rem; margin-bottom: .75rem; }
  label { display: grid; gap: .35rem; color: var(--mantis-text-muted); font-size: .78rem; margin: .55rem 0; }
  select, input, textarea { width: 100%; box-sizing: border-box; border: 1px solid var(--mantis-border); border-radius: 7px; background: rgba(0,0,0,.18); color: var(--mantis-text); padding: .55rem; font: inherit; }
  textarea { resize: vertical; }
  .evidence-list { list-style: none; padding: 0; margin: .5rem 0; display: grid; gap: .4rem; }
  .evidence-list li { padding: .55rem; border: 1px solid var(--mantis-border); border-radius: 6px; overflow-wrap: anywhere; font-size: .8rem; }
  .evidence-list small { display: block; color: var(--mantis-text-muted); margin-top: .25rem; }

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
  @media (max-width: 600px) { .tracking-grid { grid-template-columns: 1fr; } }

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
    position: relative;
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

  .status-pill {
    position: static;
    display: inline-flex;
    width: max-content;
    font-size: 0.65rem;
    padding: 0.16rem 0.42rem;
    border-radius: var(--radius-pill);
    border: 1px solid;
    background: var(--mantis-bg-solid);
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
    gap: 1rem;
  }

  .detail-header h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .badges {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.25rem;
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

  .steps-list {
    margin: 0;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .actions-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
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
  .preventive-guide { display:grid; grid-template-columns:64px 1fr auto; gap:.8rem; align-items:center; margin:1rem 0; padding:.7rem; border:1px solid color-mix(in srgb,var(--guide-accent) 45%,var(--mantis-border)); border-radius:10px; color:inherit; text-decoration:none; background:linear-gradient(110deg,color-mix(in srgb,var(--guide-accent) 10%,transparent),rgba(255,255,255,.015)); }
  .preventive-guide:hover { border-color:var(--guide-accent); }
  .preventive-guide img { width:64px; height:64px; border-radius:8px; object-fit:cover; background:#080a0c; }
  .preventive-guide>span { min-width:0; display:grid; gap:.18rem; }
  .preventive-guide small { color:var(--guide-accent); font:700 .62rem/1 var(--font-meta); letter-spacing:.08em; text-transform:uppercase; }
  .preventive-guide strong { font-size:.86rem; }
  .preventive-guide p { margin:0; color:var(--mantis-text-muted); font-size:.72rem; line-height:1.4; }
  .preventive-guide>b { color:var(--guide-accent); font-size:.72rem; white-space:nowrap; }
  @media (max-width:700px) { .preventive-guide { grid-template-columns:52px 1fr; } .preventive-guide>b { grid-column:2; } }

	.actions-view .list-panel,
	.actions-view .detail-panel { background:var(--ui-material-panel); }
	.actions-view .list-item { background:var(--ui-material-solid); border-color:var(--ui-border-subtle); }
	.actions-view .list-item:hover { border-color:var(--ui-border-default); }
	.actions-view .list-item.active { border-color:color-mix(in srgb,var(--ui-accent) 34%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-accent) 7%,var(--ui-material-solid)); box-shadow:inset 2px 0 0 var(--ui-accent); }
	.actions-view select,
	.actions-view input,
	.actions-view textarea,
	.actions-view .evidence-list li { background:var(--ui-material-solid); border-color:var(--ui-border-subtle); }
	.actions-view .preventive-guide { background:var(--ui-material-solid); }
</style>
