<script lang="ts">
  import { page } from '$app/stores';
  import { open } from '@tauri-apps/plugin-shell';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { guides, getGuide, type Guide } from '$lib/guides';
  import { t } from '$lib/i18n';

  let search = $state('');
  let category = $state('Tous');
  let linkError = $state<string | null>(null);
  const categories = ['Tous', ...new Set(guides.map((guide) => guide.category))];
  const selectedId = $derived($page.url.searchParams.get('id') ?? guides[0].id);
  const selected = $derived(getGuide(selectedId) ?? guides[0]);
  const filteredGuides = $derived(guides.filter((guide) => {
    const query = search.trim().toLocaleLowerCase('fr');
    const matchesCategory = category === 'Tous' || guide.category === category;
    const haystack = `${guide.title} ${guide.summary} ${guide.tags.join(' ')}`.toLocaleLowerCase('fr');
    return matchesCategory && (!query || haystack.includes(query));
  }));

  async function openExternal(event: MouseEvent, url: string) {
    event.preventDefault();
    if (!/^https:\/\//i.test(url)) return;
    try { await open(url); linkError = null; }
    catch (error) { linkError = `${t('Impossible d’ouvrir cette ressource :')} ${String(error)}`; }
  }

  function guideHref(guide: Guide): string {
    return `/guides?id=${guide.id}`;
  }
</script>

<section class="wf-view guide-view" style={`--guide-accent:${selected.accent}`}>
  <GuideHeader
    title="Guides de protection"
    question="Quelle mesure réduit réellement le risque que je viens d’observer ?"
    intro="Des parcours courts, sourcés et réalistes. Choisissez une mesure, comprenez ses limites, puis appliquez-la à votre rythme."
  />

  <div class="resource-ribbon" aria-label={t('Ressources publiques recommandées')}>
    <span class="resource-mark">{t('Ressources publiques')}</span>
    <a href="https://www.privacyguides.org/en/" onclick={(event) => openExternal(event, 'https://www.privacyguides.org/en/')}><strong>Privacy Guides</strong><span>{t('Choix techniques maintenus')}</span><b>↗</b></a>
    <a href="https://exitchatcontrol.org/" onclick={(event) => openExternal(event, 'https://exitchatcontrol.org/')}><strong>Exit Chat Control</strong><span>{t('Manuel de souveraineté numérique')}</span><b>↗</b></a>
  </div>
  {#if linkError}<p class="link-error" role="alert">{linkError}</p>{/if}

  <div class="guide-shell">
    <aside class="catalog" aria-label={t('Catalogue des guides')}>
      <div class="catalog-tools">
        <label><span>{t('Rechercher')}</span><input type="search" bind:value={search} placeholder={t('Alias, Signal, Linux…')} /></label>
        <div class="category-tabs" aria-label={t('Filtrer par catégorie')}>
          {#each categories as item}<button class:active={category === item} onclick={() => category = item}>{t(item)}</button>{/each}
        </div>
      </div>
      <div class="catalog-count">{t(`${filteredGuides.length} parcours`)}</div>
      {#if filteredGuides.length}
        <nav class="guide-list">
          {#each filteredGuides as guide (guide.id)}
            <a href={guideHref(guide)} class:active={guide.id === selected.id} style={`--item-accent:${guide.accent}`}>
              <img src={guide.image} alt="" />
              <span class="guide-list-copy"><small>{t(guide.category)} · {guide.minutes} min</small><strong>{t(guide.shortTitle)}</strong><span>{t(guide.summary)}</span></span>
              <b aria-hidden="true">›</b>
            </a>
          {/each}
        </nav>
      {:else}
        <EmptyState symbol="⌕" title="Aucun parcours" message="Essayez un terme ou une catégorie plus générale." />
      {/if}
    </aside>

    <article class="guide-detail">
      <header class="guide-hero">
        <img src={selected.image} alt={t(selected.imageAlt)} />
        <div class="hero-shade"></div>
        <div class="hero-copy">
          <div class="meta"><span>{t(selected.category)}</span><span>{t(selected.level)}</span><span>{selected.minutes} min</span></div>
          <h1>{t(selected.title)}</h1>
          <p>{t(selected.summary)}</p>
        </div>
      </header>

      <div class="guide-body">
        <section class="outcome-grid">
          <div><small>{t('À utiliser quand')}</small><p>{t(selected.when)}</p></div>
          <div><small>{t('Résultat visé')}</small><p>{t(selected.outcome)}</p></div>
        </section>

        <section class="steps-section">
          <div class="section-heading"><span>{t('PARCOURS')}</span><h2>{t('Passer de l’intention à l’action')}</h2></div>
          <ol class="premium-steps">
            {#each selected.steps as step, index}
              <li><span>{String(index + 1).padStart(2, '0')}</span><div><h3>{t(step.title)}</h3><p>{t(step.detail)}</p></div></li>
            {/each}
          </ol>
        </section>

        {#if selected.gallery}
          <section class="gallery" aria-label={t('Aperçus des applications')}>
            {#each selected.gallery as image}<figure><img src={image.src} alt={t(image.alt)} /><figcaption>{t(image.alt)}</figcaption></figure>{/each}
          </section>
        {/if}

        <div class="verify-grid">
          <section class="check-panel">
            <div class="section-heading"><span>{t('CONTRÔLE')}</span><h2>{t('Checklist de sortie')}</h2></div>
            <ul>{#each selected.checklist as item}<li><span>✓</span>{t(item)}</li>{/each}</ul>
          </section>
          <section class="limits-panel">
            <div class="section-heading"><span>{t('LIMITES')}</span><h2>{t('Ce que cela ne garantit pas')}</h2></div>
            <ul>{#each selected.cautions as item}<li>{t(item)}</li>{/each}</ul>
          </section>
        </div>

        <section class="resources-section">
          <div class="section-heading"><span>{t('ALLER PLUS LOIN')}</span><h2>{t('Ressources vérifiées')}</h2></div>
          <div class="resource-cards">
            {#each selected.resources as resource}
              <a class:featured={resource.featured} href={resource.url} onclick={(event) => openExternal(event, resource.url)}>
                <span><strong>{t(resource.label)}</strong><small>{t(resource.note)}</small></span><b>↗</b>
              </a>
            {/each}
          </div>
        </section>

        <footer class="guide-next">
          <span><small>{t('Étape suivante')}</small><strong>{t('Appliquez la mesure, puis revenez noter ce qui a changé.')}</strong></span>
          <a class="wf-btn primary" href={selected.relatedHref ?? '/actions'}>{t('Ouvrir le module lié →')}</a>
        </footer>
      </div>
    </article>
  </div>
</section>

<style>
  .guide-view{max-width:1500px}.resource-ribbon{display:grid;grid-template-columns:auto 1fr 1fr;align-items:stretch;margin:0 0 1rem;border:1px solid var(--ui-border-subtle);border-radius:12px;overflow:hidden;background:rgba(8,10,13,.52)}
  .resource-mark{display:flex;align-items:center;padding:.75rem 1rem;color:var(--guide-accent);font:750 .66rem/1 var(--font-meta);letter-spacing:.11em;text-transform:uppercase;border-right:1px solid var(--ui-border-subtle)}
  .resource-ribbon a{display:grid;grid-template-columns:1fr auto;gap:.12rem .8rem;padding:.68rem 1rem;color:var(--ui-text);text-decoration:none;border-right:1px solid var(--ui-border-subtle);transition:background .2s}.resource-ribbon a:hover{background:rgba(255,255,255,.045)}.resource-ribbon strong{font-size:.82rem}.resource-ribbon span:not(.resource-mark){grid-row:2;color:var(--ui-text-tertiary);font-size:.68rem}.resource-ribbon b{grid-row:1/3;align-self:center;color:var(--guide-accent)}.link-error{padding:.7rem;border:1px solid var(--mantis-danger);border-radius:8px;color:var(--mantis-danger);font-size:.78rem}
  .guide-shell{display:grid;grid-template-columns:minmax(280px,340px) minmax(0,1fr);gap:1rem;align-items:start}.catalog,.guide-detail{border:1px solid var(--ui-border-subtle);border-radius:14px;background:linear-gradient(145deg,rgba(20,23,28,.92),rgba(9,11,14,.92));box-shadow:0 20px 60px rgba(0,0,0,.18)}.catalog{position:sticky;top:1rem;max-height:calc(100vh - 2rem);overflow:auto;padding:.8rem}.catalog-tools label{display:grid;gap:.35rem}.catalog-tools label span,.catalog-count{font:700 .62rem/1 var(--font-meta);letter-spacing:.1em;text-transform:uppercase;color:var(--ui-text-tertiary)}.catalog-tools input{min-height:38px;padding:.55rem .7rem}.category-tabs{display:flex;gap:.35rem;overflow:auto;padding:.65rem 0}.category-tabs button{padding:.42rem .6rem;border:1px solid var(--ui-border-subtle);border-radius:999px;background:transparent;color:var(--ui-text-tertiary);font-size:.66rem;white-space:nowrap}.category-tabs button.active{border-color:var(--guide-accent);color:var(--ui-text);background:color-mix(in srgb,var(--guide-accent) 13%,transparent)}.catalog-count{padding:.3rem .2rem .7rem}.guide-list{display:grid;gap:.42rem}.guide-list>a{position:relative;display:grid;grid-template-columns:58px 1fr auto;gap:.7rem;align-items:center;padding:.55rem;border:1px solid transparent;border-radius:10px;color:inherit;text-decoration:none;transition:.2s}.guide-list>a:hover{background:rgba(255,255,255,.035)}.guide-list>a.active{border-color:color-mix(in srgb,var(--item-accent) 45%,transparent);background:color-mix(in srgb,var(--item-accent) 10%,transparent)}.guide-list img{width:58px;height:58px;border-radius:8px;object-fit:cover;background:#0a0c0f}.guide-list-copy{min-width:0;display:grid;gap:.16rem}.guide-list-copy small{color:var(--item-accent);font:650 .58rem/1 var(--font-meta);text-transform:uppercase;letter-spacing:.06em}.guide-list-copy strong{font-size:.8rem}.guide-list-copy>span{display:-webkit-box;overflow:hidden;line-clamp:2;-webkit-line-clamp:2;-webkit-box-orient:vertical;color:var(--ui-text-tertiary);font-size:.66rem;line-height:1.35}.guide-list>a>b{color:var(--ui-text-tertiary);font-size:1.2rem}
  .guide-detail{overflow:hidden}.guide-hero{position:relative;min-height:370px;display:flex;align-items:end;overflow:hidden}.guide-hero>img{position:absolute;inset:0;width:100%;height:100%;object-fit:cover;background:#090b0e}.hero-shade{position:absolute;inset:0;background:linear-gradient(90deg,rgba(5,7,9,.94) 0%,rgba(5,7,9,.62) 54%,rgba(5,7,9,.14)),linear-gradient(0deg,rgba(5,7,9,.95),transparent 62%)}.hero-copy{position:relative;z-index:1;max-width:720px;padding:2.4rem}.meta{display:flex;gap:.45rem}.meta span{padding:.34rem .55rem;border:1px solid color-mix(in srgb,var(--guide-accent) 50%,transparent);border-radius:999px;color:var(--guide-accent);background:rgba(0,0,0,.3);font:700 .62rem/1 var(--font-meta);letter-spacing:.06em;text-transform:uppercase}.hero-copy h1{max-width:650px;margin:.9rem 0 .7rem;font-size:clamp(2rem,4.1vw,3.7rem);line-height:1.02;letter-spacing:-.055em}.hero-copy p{max-width:670px;margin:0;color:#c6cbd2;font-size:1rem;line-height:1.55}.guide-body{padding:1.45rem 2rem 2rem}.outcome-grid{display:grid;grid-template-columns:1fr 1fr;gap:1rem}.outcome-grid>div{padding:1rem;border:1px solid var(--ui-border-subtle);border-radius:10px;background:rgba(255,255,255,.018)}.outcome-grid small,.section-heading>span,.guide-next small{color:var(--guide-accent);font:750 .62rem/1 var(--font-meta);letter-spacing:.11em;text-transform:uppercase}.outcome-grid p{margin:.55rem 0 0;color:var(--ui-text-secondary);font-size:.8rem;line-height:1.5}.steps-section,.resources-section{padding-top:2rem}.section-heading h2{margin:.3rem 0 1rem;font-size:1.18rem;letter-spacing:-.025em}.premium-steps{list-style:none;padding:0;margin:0;display:grid;grid-template-columns:1fr 1fr;gap:.65rem}.premium-steps li{display:grid;grid-template-columns:42px 1fr;gap:.75rem;padding:1rem;border:1px solid var(--ui-border-subtle);border-radius:10px;background:linear-gradient(135deg,rgba(255,255,255,.027),transparent)}.premium-steps li>span{color:var(--guide-accent);font:600 1.15rem/1 var(--font-meta)}.premium-steps h3{margin:0 0 .35rem;font-size:.86rem}.premium-steps p{margin:0;color:var(--ui-text-tertiary);font-size:.75rem;line-height:1.48}.gallery{display:grid;grid-template-columns:repeat(3,1fr);gap:.65rem;margin-top:1.5rem}.gallery figure{position:relative;min-height:200px;margin:0;overflow:hidden;border:1px solid var(--ui-border-subtle);border-radius:10px;background:#080a0c}.gallery img{width:100%;height:250px;object-fit:contain}.gallery figcaption{position:absolute;inset:auto 0 0;padding:.55rem;background:linear-gradient(transparent,rgba(0,0,0,.9));color:#d4d7da;font-size:.65rem}.verify-grid{display:grid;grid-template-columns:1fr 1fr;gap:.8rem;padding-top:2rem}.check-panel,.limits-panel{padding:1rem;border:1px solid var(--ui-border-subtle);border-radius:10px}.check-panel ul,.limits-panel ul{list-style:none;padding:0;margin:0;display:grid;gap:.55rem}.check-panel li,.limits-panel li{display:flex;gap:.6rem;color:var(--ui-text-secondary);font-size:.76rem;line-height:1.45}.check-panel li span{color:var(--guide-accent)}.limits-panel{background:rgba(238,167,86,.035)}.limits-panel li::before{content:'—';color:#d7a168}.resource-cards{display:grid;grid-template-columns:repeat(2,1fr);gap:.6rem}.resource-cards a{display:flex;justify-content:space-between;gap:1rem;padding:.9rem;border:1px solid var(--ui-border-subtle);border-radius:9px;color:inherit;text-decoration:none;background:rgba(255,255,255,.018)}.resource-cards a:hover{border-color:var(--guide-accent)}.resource-cards a.featured{background:color-mix(in srgb,var(--guide-accent) 7%,transparent)}.resource-cards a>span{display:grid;gap:.3rem}.resource-cards strong{font-size:.8rem}.resource-cards small{color:var(--ui-text-tertiary);font-size:.68rem;line-height:1.4}.resource-cards b{color:var(--guide-accent)}.guide-next{display:flex;justify-content:space-between;gap:1rem;align-items:center;margin-top:1.6rem;padding-top:1.2rem;border-top:1px solid var(--ui-border-subtle)}.guide-next>span{display:grid;gap:.35rem}.guide-next strong{font-size:.78rem}
  @media(max-width:980px){.guide-shell{grid-template-columns:1fr}.catalog{position:static;max-height:none}.guide-list{grid-template-columns:repeat(2,1fr)}.guide-hero{min-height:320px}.resource-ribbon{grid-template-columns:1fr 1fr}.resource-mark{grid-column:1/-1;border-bottom:1px solid var(--ui-border-subtle)}}
  @media(max-width:680px){.guide-list,.premium-steps,.verify-grid,.outcome-grid,.resource-cards,.gallery{grid-template-columns:1fr}.guide-body,.hero-copy{padding:1.2rem}.resource-ribbon{grid-template-columns:1fr}.resource-ribbon a{border-top:1px solid var(--ui-border-subtle)}.guide-next{align-items:stretch;flex-direction:column}.guide-hero{min-height:390px}}
</style>
