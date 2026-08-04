<script lang="ts">
  import { onMount } from 'svelte';
  import { getExposure, listRemediationRecommendations, createRemediationPlan, enrichRemediationPlan, type Exposure, type RemediationRecommendation, type RemediationPlan, type RemediationAiEnrichment } from '$lib/api';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import RemediationJourney from '$lib/components/RemediationJourney.svelte';
  import NextStepBar from '$lib/NextStepBar.svelte';
  import { recommendGuideForContext } from '$lib/guides';
  import { t } from '$lib/i18n';

  let exposure = $state<Exposure | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let recommendations = $state<RemediationRecommendation[]>([]);
  let plan = $state<RemediationPlan | null>(null);
  let planning = $state(false);
  let enriching = $state(false);
  let enrichment = $state<RemediationAiEnrichment | null>(null);
  const preventiveGuide = $derived(exposure ? recommendGuideForContext(`${exposure.title} ${exposure.kind} ${exposure.what} ${exposure.why}`) : null);

  onMount(() => {
    const unsub = page.subscribe($page => {
      const id = $page.params.id;
      if (id) {
        loadExposure(id);
      } else {
        exposure = null;
        loading = false;
        error = null;
      }
    });
    return unsub;
  });

  async function loadExposure(id: string) {
    try {
      exposure = await getExposure(id);
      recommendations = await listRemediationRecommendations(exposure.kind);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function createPlan() {
    if (!exposure) return;
    planning = true; error = null;
    try { plan = await createRemediationPlan(exposure.id); }
    catch (e) { error = String(e); }
    finally { planning = false; }
  }

  async function enrichPlan() {
    if (!plan) return;
    enriching = true; error = null;
    try { enrichment = await enrichRemediationPlan(plan.id); }
    catch (e) { error = String(e); }
    finally { enriching = false; }
  }

  function getSeverityLabel(sev: string): string {
    return sev;
  }

  function getStatusLabel(status: string): string {
    const map: Record<string, string> = {
      nouvelle: 'Nouvelle',
      en_suivi: 'En suivi',
      acceptee: 'Acceptée',
      reduite: 'Réduite'
    };
    return map[status] ?? status;
  }

  function getSeverityTone(sev: string): 'danger' | 'warning' | 'success' | 'neutral' {
    switch (sev) {
      case 'critique': return 'danger';
      case 'élevée': return 'warning';
      case 'modérée': return 'warning';
      case 'faible': return 'success';
      default: return 'neutral';
    }
  }

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleDateString('fr-FR');
    } catch {
      return dateStr;
    }
  }
</script>

<section class="wf-view exposure-detail-view">
  <GuideHeader title="Détail de l’exposition" question="Que démontre ce constat et comment réduire le risque ?" intro="Vérifiez l’observation, choisissez un plan puis suivez une preuve de réalisation. La décision reste toujours la vôtre." />
  <RemediationJourney current="exposition" />

  {#if loading}
    <StatePanel tone="info" title="Chargement de l’exposition" message="Lecture du constat et des recommandations locales…" />
  {:else if error}
    <StatePanel tone="danger" title="Exposition indisponible" message={error} />
  {:else if exposure}
    <article class="glass-card exposure-record">
      <header class="record-header">
        <div><a href="/expositions" class="back-link">← Toutes les expositions</a><h2>{t(exposure.title)}</h2><p>{exposure.kind} · détectée le {formatDate(exposure.discovered_at)}</p></div>
        <div class="record-badges"><StatusBadge label={getSeverityLabel(exposure.severity)} tone={getSeverityTone(exposure.severity)} dot /><StatusBadge label={getStatusLabel(exposure.status)} tone="accent" /></div>
      </header>

      <div class="evidence-grid">
        <section><span class="section-label">Observation</span><h3>Ce qui a été constaté</h3><p>{exposure.what}</p></section>
        <section><span class="section-label">Interprétation</span><h3>Pourquoi cela mérite attention</h3><p>{exposure.why}</p></section>
      </div>
      <div class="source-strip"><div><span>Source enregistrée</span><strong>{exposure.source}</strong></div>{#if exposure.folder_id}<div><span>Dossier associé</span><strong>{exposure.folder_id}</strong></div>{/if}<small>Cette source documente le constat ; elle ne prouve pas automatiquement que le profil vous appartient.</small></div>

      {#if preventiveGuide}
        <a class="exposure-guide" href={`/guides?id=${preventiveGuide.id}`} style={`--guide-accent:${preventiveGuide.accent}`}>
          <img src={preventiveGuide.image} alt="" />
          <span><small>Réduire le risque à la source</small><strong>{preventiveGuide.shortTitle}</strong><p>{preventiveGuide.outcome}</p></span>
          <b>Voir le guide →</b>
        </a>
      {/if}
    </article>

    <article class="glass-card remediation-panel">
      <div class="remediation-heading"><div><p class="eyebrow">Étape suivante</p><h2>Choisir une réduction vérifiable</h2><p>Les recommandations sont locales et déterministes. Examinez la preuve attendue avant de créer le plan.</p></div><button class="wf-btn primary" disabled={planning || !recommendations.length || Boolean(plan)} onclick={createPlan}>{planning ? 'Création…' : plan ? 'Plan créé ✓' : 'Créer le plan d’action'}</button></div>
      {#if recommendations.length}
        <div class="recommendations">{#each recommendations as recommendation, index}<article><div class="recommendation-meta"><span>{String(index + 1).padStart(2, '0')}</span><div><strong>{recommendation.title}</strong><small>{recommendation.priority} · {recommendation.execution_mode}</small></div></div><p>{recommendation.why}</p><ol>{#each recommendation.steps as step}<li>{step}</li>{/each}</ol><div class="proof-target"><span>Preuve attendue</span><strong>{recommendation.proof_expected}</strong><small>Résultat visé : {recommendation.expected_outcome}</small></div></article>{/each}</div>
      {:else}<StatePanel compact title="Aucune recommandation disponible" message="Aucun modèle local n’est encore défini pour ce type d’exposition." />{/if}

      {#if plan}<div class="plan-created"><div><strong>Plan enregistré</strong><span>{plan.items.length} action(s) créée(s) · priorité {plan.priority}</span></div><a class="wf-btn primary" href="/actions">Suivre les actions →</a><button class="wf-btn" disabled={enriching} onclick={enrichPlan}>{enriching ? 'Analyse locale…' : enrichment ? 'Rafraîchir l’explication' : 'Expliquer la priorité'}</button></div>{/if}
      {#if enrichment}<details class="secondary-section" open><summary>Explication locale du plan <span>{enrichment.mode === 'ia_locale' ? `IA locale · ${enrichment.model_label ?? 'modèle local'}` : 'Présentation déterministe'}</span></summary><div class="ai-enrichment"><p>{enrichment.summary}</p><strong>Pourquoi cette priorité</strong><p>{enrichment.priority_rationale}</p><strong>À retenir</strong><ul>{#each enrichment.education as point}<li>{point}</li>{/each}</ul><strong>Précautions</strong><ul>{#each enrichment.cautions as caution}<li>{caution}</li>{/each}</ul>{#if enrichment.error_message}<small>L’IA locale n’a pas pu être utilisée : {enrichment.error_message}</small>{/if}</div></details>{/if}
    </article>

    <NextStepBar hint="Créer un plan génère des actions traçables ; cela ne transforme pas automatiquement l’exposition en incident." primaryHref="/actions" primaryLabel="Voir les actions"><a class="wf-btn" href="/incidents">Voir les points à traiter</a></NextStepBar>
  {/if}
</section>

<style>
  .exposure-detail-view { max-width:1200px; }
  .exposure-record { padding:0; overflow:hidden; }
  .record-header { display:flex; justify-content:space-between; gap:1rem; padding:1.1rem 1.2rem; border-bottom:1px solid var(--ui-border-subtle); }
  .back-link { color:var(--ui-link); font-size:.72rem; text-decoration:none; }.record-header h2 { margin:.45rem 0 .2rem; font-size:1.35rem; letter-spacing:-.03em; }.record-header p { margin:0; color:var(--ui-text-secondary); font-size:.76rem; }.record-badges { display:flex; align-items:flex-start; flex-wrap:wrap; gap:.4rem; }
  .evidence-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); }.evidence-grid section { min-height:150px; padding:1.15rem 1.2rem; }.evidence-grid section+section { border-left:1px solid var(--ui-border-subtle); }.section-label { color:var(--ui-link); font:700 .66rem/1 var(--font-meta); letter-spacing:.09em; text-transform:uppercase; }.evidence-grid h3 { margin:.45rem 0; font-size:.9rem; }.evidence-grid p { margin:0; color:var(--ui-text-secondary); font-size:.82rem; line-height:1.55; }
  .source-strip { display:grid; grid-template-columns:minmax(180px,.8fr) minmax(180px,.8fr) minmax(260px,1.4fr); gap:.8rem; padding:.85rem 1.2rem; border-top:1px solid var(--ui-border-subtle); background:var(--ui-surface-2); }.source-strip span,.source-strip strong { display:block; }.source-strip span { color:var(--ui-text-tertiary); font-size:.65rem; text-transform:uppercase; letter-spacing:.07em; }.source-strip strong { margin-top:.2rem; font-size:.75rem; overflow-wrap:anywhere; }.source-strip small { align-self:center; color:var(--ui-text-tertiary); font-size:.68rem; line-height:1.4; }
  .exposure-guide { display:grid; grid-template-columns:66px 1fr auto; gap:.8rem; align-items:center; margin:1rem 1.2rem; padding:.7rem; border:1px solid color-mix(in srgb,var(--guide-accent) 45%,var(--ui-border-default)); border-radius:var(--radius-sm); color:inherit; text-decoration:none; background:linear-gradient(110deg,color-mix(in srgb,var(--guide-accent) 9%,transparent),transparent); }.exposure-guide:hover { border-color:var(--guide-accent); }.exposure-guide img { width:66px; height:66px; border-radius:7px; object-fit:cover; background:#080a0c; }.exposure-guide>span { display:grid; gap:.16rem; min-width:0; }.exposure-guide small { color:var(--guide-accent); font:700 .61rem/1 var(--font-meta); letter-spacing:.08em; text-transform:uppercase; }.exposure-guide strong { font-size:.84rem; }.exposure-guide p { margin:0; color:var(--ui-text-secondary); font-size:.72rem; line-height:1.4; }.exposure-guide>b { color:var(--guide-accent); font-size:.72rem; white-space:nowrap; }
  .remediation-heading { display:flex; justify-content:space-between; gap:1rem; align-items:flex-start; }.remediation-heading h2 { margin:.2rem 0 .3rem; font-size:1.15rem; }.remediation-heading p:not(.eyebrow) { margin:0; max-width:48rem; color:var(--ui-text-secondary); font-size:.8rem; }.remediation-heading .wf-btn { flex:0 0 auto; }
  .recommendations { display:grid; grid-template-columns:repeat(auto-fit,minmax(270px,1fr)); gap:.7rem; margin-top:1rem; }.recommendations>article { padding:.9rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-sm); background:var(--ui-surface-2); }.recommendation-meta { display:grid; grid-template-columns:auto 1fr; gap:.65rem; }.recommendation-meta>span { color:var(--ui-link); font:700 .68rem/1.4 var(--font-meta); }.recommendation-meta strong,.recommendation-meta small { display:block; }.recommendation-meta strong { font-size:.82rem; }.recommendation-meta small { margin-top:.15rem; color:var(--ui-text-tertiary); font-size:.66rem; text-transform:uppercase; }.recommendations>article>p,.recommendations li { color:var(--ui-text-secondary); font-size:.75rem; line-height:1.45; }.recommendations ol { padding-left:1.2rem; }.proof-target { display:grid; gap:.2rem; margin-top:.7rem; padding:.65rem; border-left:2px solid var(--ui-accent); background:color-mix(in srgb,var(--ui-accent) 5%,transparent); }.proof-target span { color:var(--ui-accent); font-size:.64rem; font-weight:700; text-transform:uppercase; }.proof-target strong { font-size:.74rem; }.proof-target small { color:var(--ui-text-tertiary); font-size:.67rem; }
  .plan-created { display:flex; align-items:center; gap:.65rem; margin-top:1rem; padding:.8rem; border:1px solid color-mix(in srgb,var(--ui-success) 42%,var(--ui-border-default)); border-radius:var(--radius-sm); background:color-mix(in srgb,var(--ui-success) 5%,var(--ui-surface-1)); }.plan-created>div { flex:1; }.plan-created strong,.plan-created span { display:block; }.plan-created span { margin-top:.15rem; color:var(--ui-text-secondary); font-size:.72rem; }.plan-created .wf-btn { margin:0; }
  .ai-enrichment { padding-top:.8rem; color:var(--ui-text-secondary); font-size:.78rem; }.ai-enrichment strong { color:var(--ui-text-primary); }.ai-enrichment ul { padding-left:1.2rem; }
  @media(max-width:700px){.record-header,.remediation-heading,.plan-created{align-items:stretch;flex-direction:column}.evidence-grid{grid-template-columns:1fr}.evidence-grid section+section{border-left:0;border-top:1px solid var(--ui-border-subtle)}.source-strip{grid-template-columns:1fr}.record-badges{order:-1}.exposure-guide{grid-template-columns:54px 1fr}.exposure-guide img{width:54px;height:54px}.exposure-guide>b{grid-column:2}}
</style>
