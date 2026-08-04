<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getFolder,
    deleteFolder,
    listIdentities,
    listExposures,
    listIncidents,
    type Folder,
    type Identity,
    type Exposure,
    type Incident
  } from '$lib/api';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import NextStepBar from '$lib/NextStepBar.svelte';

  let folderId: string | null = null;
  let folder = $state<Folder | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let deleting = $state(false);
  let deleteError = $state<string | null>(null);

  let identities = $state<Identity[]>([]);
  let exposures = $state<Exposure[]>([]);
  let incidents = $state<Incident[]>([]);
  onMount(() => {
    const unsub = page.subscribe($page => {
      folderId = $page.params.id ?? null;
      if (folderId) {
        loadFolderData();
      } else {
        folder = null;
        loading = false;
        error = null;
        identities = [];
        exposures = [];
        incidents = [];
      }
    });
    return unsub;
  });

  async function loadFolderData() {
    try {
      // Load folder
      folder = await getFolder(folderId!);
      // Load related data
      const [ids, exps, incs] = await Promise.all([
        listIdentities(),
        listExposures(),
        listIncidents()
      ]);
      // Filter by folder_id
      identities = ids.filter(i => i.folder_id === folderId);
      exposures = exps.filter(e => e.folder_id === folderId);
      incidents = incs.filter(i => i.folder_id === folderId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleDeleteFolder() {
    if (!folderId) return;
    if (!confirm(`Supprimer le dossier « ${folder?.name ?? ''} » ? Les éléments associés seront conservés sans ce classement.`)) return;
    deleting = true;
    deleteError = null;
    try {
      await deleteFolder(folderId);
      goto('/dossiers');
    } catch (err) {
      deleteError = `Erreur : ${err}`;
    } finally {
      deleting = false;
    }
  }

  function identitySummary(identity: Identity): string {
    const values = identity.values?.filter((value) => value.status === 'active').map((value) => value.value) ?? [];
    return values.slice(0, 2).join(' · ') || 'Aucune donnée active';
  }

  function severityTone(severity: string): 'danger' | 'warning' | 'success' | 'neutral' {
    return severity === 'critique' ? 'danger' : severity === 'élevée' || severity === 'modérée' ? 'warning' : severity === 'faible' ? 'success' : 'neutral';
  }

</script>

<section class="wf-view folder-detail-view">
  <GuideHeader title="Dossier" question="Quels éléments appartiennent à cette investigation ?" intro="Le dossier organise le contexte sans modifier les identités, preuves ou décisions qu’il contient." />
  {#if loading}
    <StatePanel tone="info" title="Chargement du dossier" message="Lecture du contexte et des éléments associés…" />
  {:else if error}
    <StatePanel tone="danger" title="Dossier indisponible" message={error} />
  {:else if folder}
    <article class="glass-card folder-hero">
      <div><a href="/dossiers">← Tous les dossiers</a><p class="eyebrow">Espace de travail</p><h2>{folder.name}</h2><p>{folder.context || 'Aucun contexte particulier n’a été renseigné.'}</p></div>
      <div class="folder-stats"><div><strong>{identities.length}</strong><span>identités</span></div><div><strong>{exposures.length}</strong><span>expositions</span></div><div><strong>{incidents.length}</strong><span>points à traiter</span></div></div>
    </article>

    {#if deleteError}<StatePanel compact tone="danger" title="Suppression impossible" message={deleteError} />{/if}

    <div class="folder-sections">
      <article class="glass-card collection-card">
        <header><div><p class="eyebrow">Déclaré</p><h2>Identités</h2></div><span>{identities.length}</span></header>
        {#if identities.length}<ul>{#each identities as identity (identity.id)}<li><a href={`/identites/${identity.id}`}><div><strong>{identity.label}</strong><small>{identitySummary(identity)}</small>{#if identity.notes}<p>{identity.notes}</p>{/if}</div><StatusBadge label={identity.status === 'active' ? 'Active' : 'Inactive'} tone={identity.status === 'active' ? 'success' : 'neutral'} dot /></a></li>{/each}</ul>{:else}<EmptyState symbol="◎" title="Aucune identité" message="Associez une identité existante à ce dossier depuis sa fiche." href="/identites" actionLabel="Gérer les identités" />{/if}
      </article>

      <article class="glass-card collection-card">
        <header><div><p class="eyebrow">Observé</p><h2>Expositions</h2></div><span>{exposures.length}</span></header>
        {#if exposures.length}<ul>{#each exposures as exposure (exposure.id)}<li><a href={`/expositions/${exposure.id}`}><div><strong>{exposure.title}</strong><small>{exposure.source}</small><p>{exposure.what}</p></div><StatusBadge label={exposure.severity} tone={severityTone(exposure.severity)} dot /></a></li>{/each}</ul>{:else}<EmptyState symbol="◇" title="Aucune exposition retenue" message="Les observations vérifiées apparaîtront ici après une décision explicite." href="/veille" actionLabel="Examiner les scans" />{/if}
      </article>

      <article class="glass-card collection-card wide">
        <header><div><p class="eyebrow">Décidé</p><h2>Points à traiter</h2></div><span>{incidents.length}</span></header>
        {#if incidents.length}<ul>{#each incidents as incident (incident.id)}<li><a href={`/incidents/${incident.id}`}><div><strong>{incident.title}</strong><small>Prochaine étape : {incident.next_step}</small><p>{incident.impact}</p></div><StatusBadge label={incident.severity} tone={severityTone(incident.severity)} dot /></a></li>{/each}</ul>{:else}<EmptyState symbol="△" title="Aucun point à traiter" message="Une exposition importante pourra être qualifiée ici sans création automatique." />{/if}
      </article>
    </div>

    <details class="danger-zone">
      <summary>Gestion du dossier</summary>
      <div><div><strong>Supprimer uniquement ce classement</strong><p>Les identités et données métier associées sont conservées.</p></div><button class="wf-btn danger" onclick={handleDeleteFolder} disabled={deleting}>{deleting ? 'Suppression…' : 'Supprimer le dossier'}</button></div>
    </details>

    <NextStepBar hint="Ajoutez les observations depuis Scanner afin de préserver leur source et leur date." primaryHref="/veille" primaryLabel="Examiner un scan"><a class="wf-btn" href="/identites">Gérer les identités</a></NextStepBar>
  {/if}
</section>

<style>
  .folder-detail-view{max-width:1280px}.folder-hero{display:flex;align-items:flex-end;justify-content:space-between;gap:1rem}.folder-hero a{display:inline-block;margin-bottom:.7rem;color:var(--ui-link);font-size:.72rem;text-decoration:none}.folder-hero h2{margin:.25rem 0 .35rem;font-size:1.45rem;letter-spacing:-.035em}.folder-hero>div>p:last-child{max-width:50rem;margin:0;color:var(--ui-text-secondary);font-size:.82rem}.eyebrow{margin:0;color:var(--ui-link);font:700 .66rem/1.2 var(--font-meta);letter-spacing:.1em;text-transform:uppercase}.folder-stats{display:grid;grid-template-columns:repeat(3,minmax(90px,1fr));flex:0 0 auto}.folder-stats>div{padding:.55rem .8rem;border-left:1px solid var(--ui-border-subtle)}.folder-stats strong,.folder-stats span{display:block}.folder-stats strong{font-size:1.35rem}.folder-stats span{color:var(--ui-text-tertiary);font-size:.65rem;text-transform:uppercase}.folder-sections{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1rem}.collection-card{padding:0;overflow:hidden}.collection-card.wide{grid-column:1/-1}.collection-card>header{display:flex;align-items:center;justify-content:space-between;padding:.9rem 1rem;border-bottom:1px solid var(--ui-border-subtle)}.collection-card h2{margin:.18rem 0 0}.collection-card>header>span{display:grid;place-items:center;min-width:27px;height:27px;border:1px solid var(--ui-border-default);border-radius:50%;color:var(--ui-text-secondary);font-size:.68rem}.collection-card ul{list-style:none;margin:0;padding:0}.collection-card li+li{border-top:1px solid var(--ui-border-subtle)}.collection-card li>a{display:flex;align-items:flex-start;justify-content:space-between;gap:1rem;padding:.85rem 1rem;color:inherit;text-decoration:none}.collection-card li>a:hover{background:var(--ui-surface-2)}.collection-card li>a>div{min-width:0}.collection-card strong,.collection-card small{display:block}.collection-card li strong{font-size:.82rem}.collection-card li small{margin-top:.2rem;color:var(--ui-text-tertiary);font-size:.68rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.collection-card li p{display:-webkit-box;line-clamp:2;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;margin:.35rem 0 0;color:var(--ui-text-secondary);font-size:.72rem;line-height:1.4}.danger-zone{padding:.75rem .9rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-sm);background:var(--ui-surface-1)}.danger-zone summary{cursor:pointer;color:var(--ui-text-secondary);font-size:.78rem}.danger-zone>div{display:flex;align-items:center;justify-content:space-between;gap:1rem;margin-top:.8rem;padding-top:.8rem;border-top:1px solid var(--ui-border-subtle)}.danger-zone strong{font-size:.8rem}.danger-zone p{margin:.2rem 0 0;color:var(--ui-text-tertiary);font-size:.7rem}.danger-zone .wf-btn{margin:0}@media(max-width:800px){.folder-hero{align-items:stretch;flex-direction:column}.folder-stats{width:100%}.folder-sections{grid-template-columns:1fr}.collection-card.wide{grid-column:auto}}@media(max-width:520px){.folder-stats{grid-template-columns:1fr}.folder-stats>div{border-left:0;border-top:1px solid var(--ui-border-subtle)}.danger-zone>div{align-items:stretch;flex-direction:column}}

</style>
