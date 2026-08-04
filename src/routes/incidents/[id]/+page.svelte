<script lang="ts">
  import { onMount } from 'svelte';
  import { getIncident, type Incident } from '$lib/api';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import RemediationJourney from '$lib/components/RemediationJourney.svelte';

  let incident = $state<Incident | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Unsubscribe function
  let unsubscribe: () => void;

  async function loadIncident(id: string) {
    try {
      incident = await getIncident(id);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function createAction() {
    if (incident?.id) {
      goto(`/actions/new?incidentId=${incident.id}`);
    }
  }

  onMount(() => {
    unsubscribe = page.subscribe($page => {
      const id = $page.params.id;
      if (id) {
        loadIncident(id);
      } else {
        incident = null;
        loading = false;
        error = null;
      }
    });
    return () => unsubscribe();
  });
</script>

<section class="wf-view incident-detail-view">
  <GuideHeader title="Point à traiter" question="Pourquoi cette situation mérite-t-elle une décision ?" intro="Relisez les faits, l’impact et le niveau de confiance avant de créer une action." />
  <RemediationJourney current="incident" />
  {#if loading}
    <StatePanel tone="info" title="Chargement du point à traiter" />
  {:else if error}
    <StatePanel tone="danger" title="Point à traiter indisponible" message={error} />
  {:else if incident}
    <article class="glass-card incident-record">
      <header><div><a href="/incidents">← Tous les points à traiter</a><h2>{incident.title}</h2><p>Détecté le {new Date(incident.discovered_at).toLocaleDateString('fr-FR')}</p></div><StatusBadge label={incident.severity} tone={incident.severity === 'critique' ? 'danger' : 'warning'} dot /></header>
      <div class="decision-callout"><span>Prochaine décision</span><strong>{incident.next_step}</strong></div>
      <dl class="incident-grid"><div><dt>Fait retenu</dt><dd>{incident.what}</dd></div><div><dt>Pourquoi c’est important</dt><dd>{incident.why}</dd></div><div><dt>Impact possible</dt><dd>{incident.impact}</dd></div><div><dt>Niveau de confiance</dt><dd>{incident.confidence}</dd></div></dl>
      <footer><p>Créer une action ne modifie pas la preuve d’origine et ne marque pas automatiquement ce point comme résolu.</p><div><a href="/incidents" class="wf-btn">Retour à la liste</a><button class="wf-btn primary" onclick={createAction}>Créer une action vérifiable →</button></div></footer>
    </article>
  {/if}
</section>

<style>
  .incident-detail-view { max-width:1100px; }.incident-record { padding:0; overflow:hidden; }.incident-record>header { display:flex; justify-content:space-between; gap:1rem; padding:1.1rem 1.2rem; border-bottom:1px solid var(--ui-border-subtle); }.incident-record header a { color:var(--ui-link); font-size:.72rem; text-decoration:none; }.incident-record h2 { margin:.45rem 0 .2rem; font-size:1.3rem; letter-spacing:-.03em; }.incident-record header p { margin:0; color:var(--ui-text-secondary); font-size:.74rem; }.incident-record>.decision-callout { margin:1rem 1.2rem; }.incident-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); margin:0; border-top:1px solid var(--ui-border-subtle); }.incident-grid>div { min-height:130px; padding:1rem 1.2rem; border-bottom:1px solid var(--ui-border-subtle); }.incident-grid>div:nth-child(even) { border-left:1px solid var(--ui-border-subtle); }.incident-grid dt { color:var(--ui-text-tertiary); font-size:.66rem; text-transform:uppercase; letter-spacing:.08em; }.incident-grid dd { margin:.45rem 0 0; color:var(--ui-text-secondary); font-size:.82rem; line-height:1.55; }.incident-record footer { display:flex; align-items:center; justify-content:space-between; gap:1rem; padding:1rem 1.2rem; }.incident-record footer p { max-width:34rem; margin:0; color:var(--ui-text-tertiary); font-size:.7rem; line-height:1.4; }.incident-record footer>div { display:flex; gap:.5rem; }.incident-record footer .wf-btn { margin:0; }
  @media(max-width:700px){.incident-grid{grid-template-columns:1fr}.incident-grid>div:nth-child(even){border-left:0}.incident-record footer{align-items:stretch;flex-direction:column}.incident-record footer>div{flex-direction:column}}
</style>
