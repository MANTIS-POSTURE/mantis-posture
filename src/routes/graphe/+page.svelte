<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import cytoscape, { type Core } from 'cytoscape';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import NextStepBar from '$lib/NextStepBar.svelte';
  import { getOsintGraph, type OsintGraph, type OsintGraphEdge } from '$lib/api';
	import { activeIdentityId } from '$lib/active-identity';
	import StatePanel from '$lib/components/StatePanel.svelte';
	import PageToolbar from '$lib/components/PageToolbar.svelte';
	import SegmentedControl from '$lib/components/SegmentedControl.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import StatusBadge from '$lib/components/StatusBadge.svelte';

  let graph = $state<OsintGraph | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let container = $state<HTMLDivElement>();
  let cy: Core | null = null;
  let view = $state<'graph'|'list'|'timeline'>('graph');
  let level = $state('all');
  let nodeType = $state('all');
  let timelineType = $state('all');
  let selected = $state<OsintGraphEdge | null>(null);
  const viewOptions = [{ value:'graph', label:'Vue visuelle' }, { value:'list', label:'Liste' }, { value:'timeline', label:'Historique' }];

  const visibleEdges = $derived(graph?.edges.filter((edge) => level === 'all' || edge.level === level) ?? []);
  const visibleNodeIds = $derived(new Set(visibleEdges.flatMap((edge) => [edge.from, edge.to])));
  const visibleNodes = $derived(graph?.nodes.filter((node) => (nodeType === 'all' || node.node_type === nodeType) && (visibleNodeIds.has(node.id) || graph?.edges.length === 0)) ?? []);
  const visibleTimeline = $derived(graph?.timeline.filter((item) => timelineType === 'all' || item.date_kind === timelineType) ?? []);
  const labels = $derived(new Map(graph?.nodes.map((node) => [node.id, node.label]) ?? []));

  async function refresh() {
    loading = true;
    try { graph = await getOsintGraph($activeIdentityId); }
    catch (e) { error = String(e); }
    finally { loading = false; setTimeout(renderGraph); }
	}
  onMount(() => activeIdentityId.subscribe(() => { void refresh(); }));
  onDestroy(() => cy?.destroy());

  $effect(() => { level; nodeType; view; if (!loading && view === 'graph') setTimeout(renderGraph); });

  function renderGraph() {
    if (!container || !graph || view !== 'graph') return;
    cy?.destroy();
    const allowed = new Set(visibleNodes.map((node) => node.id));
    cy = cytoscape({
      container,
      elements: [
        ...visibleNodes.map((node) => ({ data: { id: node.id, label: node.label, type: node.node_type } })),
        ...visibleEdges.filter((edge) => allowed.has(edge.from) && allowed.has(edge.to)).map((edge) => ({ data: { id: edge.id, source: edge.from, target: edge.to, label: relationLabel(edge.relation_type), level: edge.level } }))
      ],
      style: [
        { selector: 'node', style: { 'label':'data(label)','font-size':10,'font-weight':600,'color':'#e6edf3','text-wrap':'wrap','text-max-width':'110px','background-color':'#27323b','border-width':2,'border-color':'#718491','width':40,'height':40 } },
        { selector: 'node[type="identite"]', style: { 'background-color':'#167f73','border-color':'#2dd4bf','shape':'round-rectangle','width':54,'height':44 } },
        { selector: 'node[type="source"]', style: { 'background-color':'#394787','border-color':'#818cf8','shape':'diamond' } },
        { selector: 'edge', style: { 'curve-style': 'bezier', 'target-arrow-shape': 'triangle', 'width': 2, 'line-color': '#667386', 'target-arrow-color': '#667386', 'label': 'data(label)', 'font-size': 8, 'color': '#aeb8c7', 'text-background-color': '#111923', 'text-background-opacity': .85, 'text-background-padding': '2px' } },
        { selector: 'edge[level="possible"]', style: { 'line-style': 'dashed' } },
        { selector: 'edge[level="probable"]', style: { 'line-color': '#d39a42', 'target-arrow-color': '#d39a42' } },
        { selector: 'edge[level="corroboree"]', style: { 'line-color': '#43b58d', 'target-arrow-color': '#43b58d', 'width': 3 } },
        { selector: 'edge[level="contradiction"]', style: { 'line-color': '#df6672', 'target-arrow-color': '#df6672', 'line-style': 'dashed', 'width': 3 } }
      ],
      layout: { name: 'cose', animate: false, fit: true, padding: 35, nodeRepulsion: () => 9000 }
    });
    cy.on('tap', 'edge', (event) => { selected = graph?.edges.find((edge) => edge.id === event.target.id()) ?? null; });
  }

  function relationLabel(value: string) { return ({ observe:'observé pour', collecte_par:'collecté par', correspondance_multi_source:'multi-source', contredit:'contredit' } as Record<string,string>)[value] ?? value; }
  function levelLabel(value: string) { return ({ observe:'Fait observé', possible:'Possible — à vérifier', probable:'Probable', corroboree:'Corroboré', contradiction:'Contradiction' } as Record<string,string>)[value] ?? value; }
  function formatDate(value: string) { const date = new Date(value.replace(' ', 'T') + (value.includes('Z') ? '' : 'Z')); return Number.isNaN(date.getTime()) ? value : date.toLocaleString('fr-FR'); }
</script>

<section class="wf-view">
  <GuideHeader title="Connexions et historique" question="Quels éléments se recoupent, et lesquels restent à vérifier ?" intro="Cette vue relie uniquement les éléments conservés par MANTIS. Un lien possible ne prouve jamais qu’un profil appartient à une personne." />
  {#if loading}<StatePanel tone="info" title="Préparation des connexions" message="Organisation des éléments observés, de leurs sources et de vos décisions…" />
  {:else if error}<StatePanel tone="danger" title="Connexions indisponibles" message={error} />
  {:else if graph}
    <PageToolbar label="Vues et filtres des connexions">
      <SegmentedControl options={viewOptions} value={view} label="Mode d’affichage" onChange={(value) => view = value as typeof view} />
      {#if view !== 'timeline'}<label>Niveau <select bind:value={level}><option value="all">Tous</option><option value="contradiction">Contradictions</option><option value="corroboree">Corroborés</option><option value="probable">Probables</option><option value="possible">Possibles</option><option value="observe">Observés</option></select></label>{/if}
      {#if view === 'graph'}<label>Éléments <select bind:value={nodeType}><option value="all">Tous</option><option value="identite">Identités</option><option value="observation">Observations</option><option value="source">Sources</option></select></label>{/if}
      {#if view === 'timeline'}<label>Événements <select bind:value={timelineType}><option value="all">Tous</option><option value="observation">Observations</option><option value="decision">Décisions humaines</option></select></label>{/if}
	</PageToolbar>
	<div class="legend" aria-label="Légende du graphe"><span><i class="identity"></i>Identité</span><span><i class="source"></i>Source</span><span><i class="observed"></i>Observation</span><span><i class="probable"></i>Probable</span><span><i class="confirmed"></i>Corroboré</span><span><i class="contradiction"></i>Contradiction</span></div>
    {#if graph.truncated}<StatusBadge tone="warning" dot label="Vue limitée aux 150 éléments les plus récents" />{/if}
    {#if graph.nodes.length === 0}<EmptyState title="Aucune connexion à afficher" message="Lancez une veille pour faire apparaître les éléments observés, leurs sources et vos décisions." href="/veille" actionLabel="Lancer une première veille" />
    {:else if view === 'graph'}
      <div class="graph-layout"><div class="canvas" bind:this={container} aria-label="Graphe interactif des relations OSINT"></div><aside class="details">
        {#if selected}<span class="badge level-{selected.level}">{levelLabel(selected.level)}</span><h2>{labels.get(selected.from)} → {labels.get(selected.to)}</h2><p>{selected.justification}</p><h3>Preuves liées</h3>{#each selected.evidence as proof}<article class:against={proof.role==='contradictoire'}><strong>{proof.label}</strong>{#if proof.excerpt}<p>{proof.excerpt}</p>{/if}{#if proof.source_url}<a href={proof.source_url} target="_blank" rel="noreferrer">Ouvrir la source</a>{/if}</article>{/each}
        {:else}<h2>Pourquoi ce lien ?</h2><p>Cliquez sur une arête pour afficher son niveau, sa justification et les preuves qui la soutiennent ou la contredisent.</p>{/if}
      </aside></div>
    {:else if view === 'list'}
      <div class="wf-panel"><h2>{visibleEdges.length} relation(s)</h2><ul class="relation-list">{#each visibleEdges as edge}<li><button onclick={() => selected = selected?.id === edge.id ? null : edge}><span class="badge level-{edge.level}">{levelLabel(edge.level)}</span><strong>{labels.get(edge.from)} — {relationLabel(edge.relation_type)} → {labels.get(edge.to)}</strong><small>{edge.justification}</small></button>{#if selected?.id === edge.id}<div class="proofs">{#each edge.evidence as proof}<p><b>{proof.role === 'contradictoire' ? 'Élément contradictoire' : 'Preuve'} :</b> {proof.label}{proof.excerpt ? ` — ${proof.excerpt}` : ''}</p>{/each}</div>{/if}</li>{/each}</ul></div>
    {:else}
      <div class="wf-panel"><h2>Chronologie vérifiable</h2><p class="muted">La date d’observation est distincte de la date d’une décision utilisateur. Une date inconnue n’est jamais estimée.</p><ol class="timeline">{#each visibleTimeline as item}<li><time>{formatDate(item.date)}</time><div><span class="badge level-{item.level}">{item.date_kind === 'decision' ? 'Décision humaine' : 'Date observée'}</span><strong>{item.label}</strong><small>{item.source}</small></div></li>{/each}</ol></div>
    {/if}
    <NextStepBar hint="Les relations restent des aides à la revue : confirmez ou contestez les signaux depuis Veille." primaryHref="/veille" primaryLabel="Revoir les signaux"><a class="wf-btn" href="/rapports">Voir le rapport</a></NextStepBar>
  {/if}
</section>

<style>
  label{font-size:.78rem;color:var(--mantis-text-muted)}select{margin-left:.35rem;padding:.4rem;background:var(--mantis-bg);color:var(--mantis-text);border:1px solid var(--mantis-border);border-radius:5px}.graph-layout{display:grid;grid-template-columns:minmax(0,1fr) 330px;min-height:570px;border:1px solid var(--mantis-border);border-radius:9px;overflow:hidden}.canvas{min-height:570px;background:radial-gradient(circle at center,#182330,#0e151e)}.details{padding:1rem;border-left:1px solid var(--mantis-border);background:var(--mantis-bg-raised);overflow:auto}.details h2{font-size:1rem;line-height:1.4}.details h3{font-size:.82rem;margin-top:1.3rem}.details p,.details article{font-size:.8rem;color:var(--mantis-text-muted)}.details article{padding:.65rem;border:1px solid var(--mantis-border);border-radius:6px;margin:.5rem 0}.details article.against{border-color:#9b3f49}.details article p{margin:.3rem 0}.badge{display:inline-block;padding:.2rem .45rem;border-radius:99px;font-size:.65rem;background:#283443;color:#bcc7d6}.level-corroborée,.level-corroboree{background:#183e35;color:#74d5b6}.level-probable{background:#44351c;color:#efbf6a}.level-contradiction{background:#4a2228;color:#ff8d98}.relation-list,.timeline{list-style:none;padding:0;display:grid;gap:.55rem}.relation-list li{border:1px solid var(--mantis-border);border-radius:7px;overflow:hidden}.relation-list button{width:100%;display:grid;gap:.35rem;text-align:left;padding:.75rem;border:0;background:var(--mantis-bg);color:var(--mantis-text)}.relation-list small,.timeline small{color:var(--mantis-text-muted)}.proofs{padding:.65rem .9rem;border-top:1px solid var(--mantis-border);font-size:.78rem}.timeline li{display:grid;grid-template-columns:155px 1fr;gap:1rem;padding:.75rem;border-left:2px solid var(--mantis-border)}.timeline time{font-size:.75rem;color:var(--mantis-text-muted)}.timeline div{display:grid;justify-items:start;gap:.25rem}.muted{color:var(--mantis-text-muted)}.error{color:var(--mantis-danger)}@media(max-width:850px){.graph-layout{grid-template-columns:1fr}.details{border-left:0;border-top:1px solid var(--mantis-border)}.timeline li{grid-template-columns:1fr}}
  .legend{display:flex;flex-wrap:wrap;gap:.45rem .9rem;padding:.55rem .7rem;border:1px solid var(--ui-border-subtle);border-radius:7px;background:rgba(0,0,0,.16)}.legend span{display:flex;align-items:center;gap:.35rem;color:var(--ui-text-secondary);font-size:.68rem}.legend i{width:8px;height:8px;border-radius:50%;background:#718491;box-shadow:0 0 7px currentColor}.legend .identity{color:var(--ui-accent);background:var(--ui-accent)}.legend .source{color:var(--ui-ai);background:var(--ui-ai);transform:rotate(45deg);border-radius:1px}.legend .probable{color:var(--ui-warning);background:var(--ui-warning)}.legend .confirmed{color:var(--ui-success);background:var(--ui-success)}.legend .contradiction{color:var(--ui-danger);background:var(--ui-danger)}
  .canvas{background:radial-gradient(circle at 50% 45%,color-mix(in srgb,var(--ui-link) 9%,transparent),transparent 34%),linear-gradient(rgba(255,255,255,.026) 1px,transparent 1px),linear-gradient(90deg,rgba(255,255,255,.026) 1px,transparent 1px),var(--ui-canvas-elevated)!important;background-size:auto,32px 32px,32px 32px,auto!important}.graph-layout{box-shadow:var(--shadow-2)}.details{background:var(--ui-surface-1)!important}
</style>
