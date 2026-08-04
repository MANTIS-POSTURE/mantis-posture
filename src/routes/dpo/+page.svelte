<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { open } from '@tauri-apps/plugin-shell';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import RemediationJourney from '$lib/components/RemediationJourney.svelte';
  import { getRgpdReviewStatus, listRgpdRequests, listRgpdEvidence, addRgpdEvidence, listRgpdEvents, revokeRgpdDraftValidation, saveRgpdDraftRevision, updateRgpdRequestStatus, useValidatedRgpdDraft, validateRgpdDraft, type RgpdRequest, type RgpdReviewStatus, type RgpdEvidence, type RgpdEvent } from '$lib/api';
  import { activeIdentityId } from '$lib/active-identity';

  let requests = $state<RgpdRequest[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let copied = $state(false);
  let review = $state<RgpdReviewStatus|null>(null);
  let reviewBusy = $state(false);
  let reviewNotice = $state<string|null>(null);
  let sourceChecked=$state(false),identityChecked=$state(false),recipientChecked=$state(false),contentChecked=$state(false),legalNoticeAccepted=$state(false);
  let draftText=$state('');let draftSaving=$state(false);
  let evidence=$state<RgpdEvidence[]>([]); let events=$state<RgpdEvent[]>([]);
  let evidenceKind=$state('source'); let evidenceLocator=$state(''); let evidenceDescription=$state(''); let evidenceVerified=$state(false); let evidenceBusy=$state(false);

  // Local state for status updates (not persisted yet)
  let localStatuses = $state<Record<string, string>>({});

  async function refresh() {
    loading = true;
    try {
      const data = await listRgpdRequests($activeIdentityId);
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
  }
  onMount(() => activeIdentityId.subscribe(() => { void refresh(); }));

  const selectedId = $derived($page.url.searchParams.get('id') ?? requests[0]?.id ?? null);
  const selected = $derived(requests.find((r) => r.id === selectedId));

  $effect(()=>{const id=selected?.id;if(id){review=null;reviewNotice=null;draftText=selected?.draft_preview??'';sourceChecked=identityChecked=recipientChecked=contentChecked=legalNoticeAccepted=false;Promise.all([getRgpdReviewStatus(id),listRgpdEvidence(id),listRgpdEvents(id)]).then(([status,proofs,history])=>{review=status;evidence=proofs;events=history;}).catch(e=>error=String(e));}});

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

  async function copyDraft() {
    if(!selected)return;
    try {
      const authorized=await useValidatedRgpdDraft(selected.id,'copie');
      await navigator.clipboard.writeText(authorized.text);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (e) {
      error=`Copie impossible : ${String(e)}`;
    }
  }

  async function validateDraft(){if(!selected)return;reviewBusy=true;error=null;try{review=await validateRgpdDraft(selected.id,{sourceChecked,identityChecked,recipientChecked,contentChecked,legalNoticeAccepted});localStatuses={...localStatuses,[selected.id]:'status_002'};reviewNotice='Brouillon validé. La copie et l’export local sont maintenant autorisés.';}catch(e){error=String(e)}finally{reviewBusy=false}}
  async function revokeValidation(){if(!selected)return;review=await revokeRgpdDraftValidation(selected.id);localStatuses={...localStatuses,[selected.id]:'status_001'};reviewNotice='Validation retirée. La copie et l’export sont de nouveau bloqués.';}
  async function exportDraft(){if(!selected)return;try{const result=await useValidatedRgpdDraft(selected.id,'export_texte');reviewNotice=`Brouillon exporté localement : ${result.path}`;}catch(e){error=String(e)}}
  async function saveDraft(){if(!selected)return;draftSaving=true;try{review=await saveRgpdDraftRevision(selected.id,draftText);requests=requests.map(r=>r.id===selected.id?{...r,draft_preview:draftText,status_id:'status_001'}:r);localStatuses={...localStatuses,[selected.id]:'status_001'};sourceChecked=identityChecked=recipientChecked=contentChecked=legalNoticeAccepted=false;reviewNotice='Nouvelle version enregistrée. Une nouvelle validation est nécessaire.';}catch(e){error=String(e)}finally{draftSaving=false}}

  async function addEvidence(){if(!selected||!evidenceLocator.trim())return;evidenceBusy=true;try{const item=await addRgpdEvidence(selected.id,evidenceKind,evidenceLocator.trim(),evidenceDescription.trim()||null,evidenceVerified);evidence=[item,...evidence];evidenceLocator='';evidenceDescription='';evidenceVerified=false;events=await listRgpdEvents(selected.id);reviewNotice='Preuve ajoutée au suivi local.';}catch(e){error=String(e)}finally{evidenceBusy=false}}

  async function openExternalUrl(event: MouseEvent, url: string) {
    event.preventDefault();
    if (!/^https?:\/\//i.test(url)) return;
    try { await open(url); }
    catch (e) { error = `Impossible d’ouvrir ce lien : ${String(e)}`; }
  }

  const currentStatus = $derived(selected ? localStatuses[selected.id] ?? selected.status_id : '');
</script>

<section class="wf-view">
  <GuideHeader
    title="Demandes RGPD"
    question="Comment préparer une demande claire, sans rien envoyer par erreur ?"
    intro="MANTIS vous aide à préparer et suivre votre demande. Vous la relisez, l’adaptez et l’envoyez vous-même."
  />
  <RemediationJourney current="rgpd" />

  {#if loading}
    <StatePanel tone="info" title="Préparation des demandes" message="Lecture des brouillons et validations locales…" />
  {:else if error && requests.length === 0}
    <StatePanel tone="danger" title="Demandes indisponibles" message={error} />
  {:else if requests.length === 0}
    <StatePanel title="Aucune demande préparée" message="Une démarche RGPD pourra être créée après vérification d’une source et décision explicite de votre part." />
  {:else}
    {#if error}<p class="error action-error" role="alert">{error}</p>{/if}
    <div class="split-layout">
      <!-- Liste des demandes -->
      <div class="glass-card list-panel">
        <div class="list-heading"><div><p class="eyebrow">Demander</p><h2>Mes demandes</h2></div><span>{requests.length}</span></div>
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
          
          <div class="decision-callout"><span>Processus manuel</span><strong>Vérifier la cible → valider le brouillon → l’envoyer vous-même → noter le résultat</strong><small>MANTIS ne transmet jamais la demande à votre place.</small></div>

          <a class="rgpd-guide-link" href="/guides?id=guide-rgpd">
            <span aria-hidden="true">§</span>
            <div><small>Avant l’envoi · guide pratique</small><strong>Choisir le bon droit et conserver la preuve</strong><p>Accès, rectification, effacement, opposition : vérifiez la démarche adaptée et les éléments à garder.</p></div>
            <b>Ouvrir →</b>
          </a>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Organisation cible</dt>
              <dd>{selected.target}</dd>
            </div>
            <div class="field">
              <dt>Contact publié</dt>
              <dd>{selected.dpo_contact}</dd>
            </div>
            <div class="field">
              <dt>Données concernées</dt>
              <dd>{selected.data_summary}</dd>
            </div>
          </div>

          {#if selected.source_url}
            <div class="source-panel">
              <span>Page concernée</span>
              <a href={selected.source_url} onclick={(event) => openExternalUrl(event, selected.source_url!)}>{selected.source_url}</a>
              <p>Vérifiez que la page est toujours accessible et qu’elle contient bien vos données avant d’envoyer une demande.</p>
              {#if selected.contact_source_url}<p>Le contact affiché a été trouvé sur cette page publique ; vérifiez-le avant utilisation.</p>{/if}
            </div>
          {/if}

          <div class="actions-section">
            <h4>Aperçu du brouillon</h4>
            <textarea class="draft-preview" bind:value={draftText} disabled={review?.validated} aria-label="Texte du brouillon RGPD"></textarea>
            <div class="draft-tools"><span>{draftText.length} caractères · enregistré localement uniquement</span>{#if !review?.validated}<button class="wf-btn" disabled={draftSaving||draftText.trim()===selected.draft_preview.trim()} onclick={saveDraft}>{draftSaving?'Enregistrement…':'Enregistrer une nouvelle version'}</button>{/if}</div>
          </div>

          <div class="review-panel">
            <div><h4>Relecture obligatoire</h4><span class:valid={review?.validated}>{review?.validated?'Validé par vous':review?.eligible?'À vérifier avant utilisation':'Non éligible'}</span></div>
            <p>{review?.reason ?? 'Vérification de la provenance…'}</p>
            {#if review?.eligible && !review.validated}<fieldset disabled={reviewBusy}><label><input type="checkbox" bind:checked={sourceChecked}/> J’ai ouvert la source et vérifié que la page existe encore.</label><label><input type="checkbox" bind:checked={identityChecked}/> J’ai vérifié que les données semblent bien me concerner.</label><label><input type="checkbox" bind:checked={recipientChecked}/> J’ai vérifié l’organisation et le contact destinataire.</label><label><input type="checkbox" bind:checked={contentChecked}/> J’ai relu et adapté le texte, les faits et les informations entre crochets.</label><label><input type="checkbox" bind:checked={legalNoticeAccepted}/> Je comprends que MANTIS fournit un brouillon informatif, pas un conseil juridique.</label></fieldset><button class="wf-btn primary" disabled={reviewBusy||!(sourceChecked&&identityChecked&&recipientChecked&&contentChecked&&legalNoticeAccepted)} onclick={validateDraft}>{reviewBusy?'Validation…':'Valider ce brouillon'}</button>{/if}
            {#if review?.validated}<p class="validated">Version {review.contract_version} validée le {review.reviewed_at}.</p><button class="wf-btn" onclick={revokeValidation}>Modifier / retirer ma validation</button>{/if}
          </div>
          <div class="actions-section">
            <h4>Preuves et suivi de la demande</h4>
            <p class="muted">Ajoutez uniquement des références utiles : URL, note de vérification ou preuve d’envoi/réponse. Aucun secret.</p>
            {#if evidence.length}<ul class="evidence-list">{#each evidence as item}<li><strong>{item.kind}</strong> · {item.locator}{#if item.verified}<span class="verified">Vérifiée</span>{/if}{#if item.description}<small>{item.description}</small>{/if}</li>{/each}</ul>{:else}<p class="muted">Aucune preuve complémentaire enregistrée.</p>{/if}
            <div class="evidence-grid"><label>Type<select bind:value={evidenceKind}><option value="source">Source</option><option value="identity">Identité vérifiée</option><option value="recipient">Destinataire</option><option value="content">Contenu relu</option><option value="send">Envoi manuel</option><option value="response">Réponse du site</option></select></label><label>Référence<input bind:value={evidenceLocator} maxlength="2048" placeholder="URL, note ou chemin local contrôlé" /></label></div>
            <label>Description<textarea bind:value={evidenceDescription} rows="2" placeholder="Ce que cette preuve démontre"></textarea></label><label class="checkline"><input type="checkbox" bind:checked={evidenceVerified}/> J’ai vérifié cette référence</label><button class="wf-btn" disabled={evidenceBusy||!evidenceLocator.trim()} onclick={addEvidence}>{evidenceBusy?'Ajout…':'Ajouter la preuve'}</button>
            {#if events.length}<details class="secondary-section"><summary>Historique de la demande <span>{events.length}</span></summary><ul class="event-list">{#each events as event}<li>{event.event_type} · {new Date(event.created_at).toLocaleString('fr-FR')}{#if event.note} — {event.note}{/if}</li>{/each}</ul></details>{/if}
          </div>
          {#if reviewNotice}<p class="notice" role="status">{reviewNotice}</p>{/if}

          <div class="actions-section">
            <h4>Utiliser le brouillon validé</h4>
            <div class="action-buttons">
              <button class="wf-btn" disabled={!review?.validated} onclick={copyDraft}>
                {copied ? 'Copié !' : 'Copier le brouillon'}
              </button>
              <button class="wf-btn" disabled={!review?.validated} onclick={exportDraft}>Exporter en texte</button>
              
              {#if currentStatus === 'status_002' && review?.validated}
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
              Aucun envoi automatique et aucun bouton d’envoi. La copie, l’export et chaque validation sont tracés localement.
            </p>
          </div>
        {:else}
          <StatePanel compact title="Sélectionnez une démarche" message="Son destinataire, ses preuves et son brouillon apparaîtront ici." />
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }
  .evidence-grid { display:grid; grid-template-columns: 1fr 2fr; gap:.75rem; }
  label { display:grid; gap:.3rem; color:var(--mantis-text-muted); font-size:.78rem; margin:.45rem 0; }
  select,input,textarea { width:100%; box-sizing:border-box; border:1px solid var(--mantis-border); border-radius:6px; background:rgba(0,0,0,.18); color:var(--mantis-text); padding:.5rem; font:inherit; }
  textarea { resize:vertical; }
  .checkline { display:flex; align-items:center; gap:.45rem; }
  .checkline input { width:auto; }
  .evidence-list,.event-list { list-style:none; padding:0; display:grid; gap:.4rem; }
  .evidence-list li,.event-list li { padding:.55rem; border:1px solid var(--mantis-border); border-radius:6px; font-size:.8rem; overflow-wrap:anywhere; }
  .evidence-list small { display:block; color:var(--mantis-text-muted); margin-top:.2rem; }
  .verified { color:var(--mantis-ok); margin-left:.5rem; font-size:.72rem; }
  .event-list li { color:var(--mantis-text-muted); }
  @media(max-width:700px){.evidence-grid{grid-template-columns:1fr;}}

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

  .source-panel { margin-top: 1.25rem; padding: 0.9rem 1rem; border: 1px solid var(--mantis-accent); border-radius: 8px; background: color-mix(in srgb, var(--mantis-accent) 7%, transparent); }
  .source-panel span { display: block; margin-bottom: 0.3rem; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--mantis-text-muted); }
  .source-panel a { display: block; overflow-wrap: anywhere; color: var(--mantis-accent); font-size: 0.88rem; }
  .source-panel p { margin: 0.6rem 0 0; color: var(--mantis-text-muted); font-size: 0.8rem; }

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
    width: 100%;
    min-height: 300px;
    color: var(--mantis-text);
    resize: vertical;
  }
  .draft-tools{display:flex;justify-content:space-between;align-items:center;gap:.7rem;margin-top:.55rem}.draft-tools span{font-size:.72rem;color:var(--mantis-text-muted)}
  .review-panel{margin-top:1rem;padding:1rem;border:1px solid var(--mantis-warn);border-radius:8px;background:color-mix(in srgb,var(--mantis-warn) 7%,transparent)}
  .review-panel>div{display:flex;justify-content:space-between;gap:1rem}.review-panel h4{margin:0}.review-panel span{font-size:.7rem;color:var(--mantis-warn)}.review-panel span.valid,.validated{color:var(--mantis-ok)}
  .review-panel p{font-size:.8rem;color:var(--mantis-text-muted)}.review-panel fieldset{display:grid;gap:.55rem;border:0;padding:.5rem 0 1rem}.review-panel label{display:flex;gap:.55rem;align-items:flex-start;font-size:.82rem}.review-panel input{margin-top:.15rem}.notice{padding:.7rem;border:1px solid var(--mantis-accent);border-radius:6px;color:var(--mantis-accent);overflow-wrap:anywhere}

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
  .rgpd-guide-link{display:grid;grid-template-columns:42px 1fr auto;gap:.8rem;align-items:center;margin:1rem 0;padding:.8rem;border:1px solid color-mix(in srgb,#d6b36a 48%,var(--mantis-border));border-radius:10px;color:inherit;text-decoration:none;background:linear-gradient(110deg,rgba(214,179,106,.1),rgba(255,255,255,.015))}.rgpd-guide-link:hover{border-color:#d6b36a}.rgpd-guide-link>span{display:grid;place-items:center;width:42px;height:42px;border-radius:9px;background:rgba(214,179,106,.12);color:#d6b36a;font:600 1.35rem/1 Georgia,serif}.rgpd-guide-link div{display:grid;gap:.16rem}.rgpd-guide-link small{color:#d6b36a;font:700 .61rem/1 var(--font-meta);letter-spacing:.08em;text-transform:uppercase}.rgpd-guide-link strong{font-size:.83rem}.rgpd-guide-link p{margin:0;color:var(--mantis-text-muted);font-size:.71rem;line-height:1.4}.rgpd-guide-link b{color:#d6b36a;font-size:.72rem;white-space:nowrap}
</style>
