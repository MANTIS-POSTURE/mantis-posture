<script lang="ts">
  import { onMount } from 'svelte';

  let paletteOpen = $state(true);
  let inspectorOpen = $state(true);
  let pressed = $state(false);

  onMount(() => {
    document.documentElement.dataset.materialPreview = 'true';
    return () => { delete document.documentElement.dataset.materialPreview; };
  });

  function handlePrimary() {
    pressed = true;
    window.setTimeout(() => { pressed = false; }, 320);
  }
</script>

<div class="material-lab">
  <div class="ambient ambient-cyan" aria-hidden="true"></div>
  <div class="ambient ambient-indigo" aria-hidden="true"></div>
  <div class="vignette" aria-hidden="true"></div>

  <section class="lab-frame" aria-labelledby="lab-title">
    <header class="lab-header">
      <div>
        <p class="lab-kicker">MATERIAL STUDY / 01</p>
        <h1 id="lab-title">Obsidian instrument</h1>
        <p class="lab-intro">A controlled preview of MANTIS as a desktop object: depth first, light second.</p>
      </div>
      <div class="window-material-status" aria-label="Window material status">
        <span class="status-filament"></span>
        <span><strong>Native surface</strong><small>Mica when available</small></span>
      </div>
    </header>

    <div class="lab-grid">
      <aside class="material-shell" aria-label="Prototype navigation">
        <div class="shell-mark"><span class="mark-core">M</span><span><strong>MANTIS</strong><small>PRIVATE INTELLIGENCE</small></span></div>
        <p class="section-label">Workspace</p>
        <nav class="material-nav">
          <a class="selected" href="#lens" aria-current="page"><span class="nav-glyph">◈</span><span>Posture lens</span><span class="nav-filament"></span></a>
          <a href="#evidence"><span class="nav-glyph">⌁</span><span>Evidence field</span></a>
          <a href="#actions"><span class="nav-glyph">↗</span><span>Next actions</span></a>
        </nav>
        <div class="shell-foot"><span class="tiny-dot"></span><span><strong>Local workspace</strong><small>Material preview only</small></span></div>
      </aside>

      <main class="lab-workspace">
        <div class="workspace-bar">
          <div><span class="bar-caption">ACTIVE OBJECT</span><strong>Posture / Instrument view</strong></div>
          <div class="bar-actions"><button class="button-secondary" type="button" onclick={() => inspectorOpen = !inspectorOpen}>{inspectorOpen ? 'Hide inspector' : 'Open inspector'}</button><button class="icon-button" type="button" aria-label="More material options">···</button></div>
        </div>

        <div class="hero-grid">
          <section id="lens" class="glass-surface lens-surface" aria-labelledby="lens-title">
            <div class="surface-label"><span>POSTURE LENS</span><span class="surface-state">READ-ONLY PREVIEW</span></div>
            <div class="lens-body">
              <div class="lens-instrument" aria-label="Posture score material preview">
                <div class="lens-ring ring-outer"></div><div class="lens-ring ring-middle"></div><div class="lens-ring ring-inner"></div>
                <div class="lens-ticks" aria-hidden="true"><i></i><i></i><i></i><i></i><i></i><i></i><i></i><i></i></div>
                <div class="lens-reading"><span class="reading-value">—</span><span class="reading-unit">SCORE PREVIEW</span></div>
                <span class="lens-needle" aria-hidden="true"></span>
              </div>
              <div class="lens-copy"><p class="lab-kicker">INSTRUMENT SURFACE</p><h2 id="lens-title">A black-glass lens for one answer.</h2><p>This is the material prototype, not a security result. Real posture data remains owned by the existing cockpit.</p><div class="lens-legend"><span><i class="legend-safe"></i>safe</span><span><i class="legend-review"></i>review</span><span><i class="legend-critical"></i>critical</span></div></div>
            </div>
          </section>

          <section id="evidence" class="glass-surface evidence-surface" aria-labelledby="evidence-title">
            <div class="surface-label"><span id="evidence-title">EVIDENCE FIELD</span><span class="surface-state">DEEP SURFACE</span></div>
            <div class="evidence-list"><div><span class="evidence-index">01</span><span><strong>Long text stays calm</strong><small>Deep glass, no row blur, readable contrast.</small></span></div><div><span class="evidence-index">02</span><span><strong>Light has an edge</strong><small>Rims respond to focus, not every border.</small></span></div><div><span class="evidence-index">03</span><span><strong>Depth has a hierarchy</strong><small>Canvas → shell → lens → crystal overlay.</small></span></div></div>
          </section>
        </div>

        <section id="actions" class="glass-surface action-surface" aria-labelledby="actions-title">
          <div class="surface-label"><span id="actions-title">CONTROL OBJECTS</span><span class="surface-state">INTERACTION STUDY</span></div>
          <div class="control-row"><div><p class="control-caption">PRIMARY ACTION</p><button class="button-primary" class:pressed type="button" onclick={handlePrimary}><span class="button-light"></span><span>{pressed ? 'Held briefly' : 'Review posture'}</span><span class="button-arrow">↗</span></button></div><div><p class="control-caption">SECONDARY ACTION</p><button class="button-secondary large" type="button" onclick={() => paletteOpen = !paletteOpen}>{paletteOpen ? 'Close command palette' : 'Open command palette'}</button></div></div>
        </section>

        {#if paletteOpen}
          <section class="crystal-palette glass-surface" aria-labelledby="palette-title">
            <div class="palette-top"><div><span class="lab-kicker">COMMAND PALETTE</span><h2 id="palette-title">What would you like to inspect?</h2></div><kbd>ESC</kbd></div>
            <div class="palette-search"><span>⌕</span><span>Type a command or jump to a surface</span><kbd>⌘ K</kbd></div>
            <div class="palette-commands"><button type="button"><span class="command-icon">◌</span><span><strong>Open posture lens</strong><small>Focus the current instrument</small></span><kbd>↵</kbd></button><button type="button"><span class="command-icon">⌁</span><span><strong>Show evidence field</strong><small>Read observations and provenance</small></span><kbd>2</kbd></button></div>
          </section>
        {/if}
      </main>

      {#if inspectorOpen}
        <aside class="crystal-inspector glass-surface" aria-labelledby="inspector-title">
          <div class="inspector-head"><div><span class="lab-kicker">CONTEXTUAL INSPECTOR</span><h2 id="inspector-title">Material anatomy</h2></div><button class="icon-button" type="button" aria-label="Close inspector" onclick={() => inspectorOpen = false}>×</button></div>
          <div class="inspector-preview"><span class="preview-glint"></span><strong>Crystal glass</strong><small>High transparency / restrained refraction</small></div>
          <dl class="inspector-list"><div><dt>Shell</dt><dd>smoked / 46%</dd></div><div><dt>Overlay</dt><dd>crystal / 28%</dd></div><div><dt>Blur</dt><dd>34px / saturated</dd></div><div><dt>Fallback</dt><dd>opaque obsidian</dd></div></dl>
          <p class="inspector-note">Material is structural: dense evidence remains deep and solid enough to read.</p>
        </aside>
      {/if}
    </div>
  </section>
</div>

<style>
  .material-lab { position:relative; min-height:100%; overflow:hidden; isolation:isolate; padding:clamp(1rem,3vw,3.5rem); color:#edf7f5; background:#000; }
  .ambient,.vignette { position:absolute; pointer-events:none; z-index:-1; }
  .ambient { width:58vw; height:58vw; border-radius:50%; filter:blur(100px); opacity:.32; }
  .ambient-cyan { top:-26vw; left:-16vw; background:radial-gradient(circle,rgba(79,235,226,.24),transparent 66%); }
  .ambient-indigo { right:-24vw; bottom:-30vw; background:radial-gradient(circle,rgba(80,70,170,.18),transparent 67%); }
  .vignette { inset:0; background:radial-gradient(ellipse at center,transparent 22%,rgba(0,0,0,.48) 100%); }
  .lab-frame { position:relative; max-width:1560px; min-height:calc(100vh - 7rem); margin:auto; padding:clamp(1rem,2vw,2rem); border:1px solid rgba(223,250,247,.13); border-radius:24px; background:linear-gradient(145deg,rgba(9,17,19,.44),rgba(2,4,5,.72)); box-shadow:inset 0 1px 0 rgba(255,255,255,.12),inset 0 -1px 0 rgba(0,0,0,.9),0 36px 100px rgba(0,0,0,.48); backdrop-filter:blur(28px) saturate(130%); -webkit-backdrop-filter:blur(28px) saturate(130%); }
  .lab-header,.workspace-bar,.surface-label,.palette-top,.inspector-head,.control-row { display:flex; align-items:center; justify-content:space-between; gap:1rem; }
  .lab-header { padding:.35rem .35rem 1.4rem; }
  .lab-kicker,.bar-caption,.control-caption,.surface-label,.surface-state,.section-label { color:#7caaa9; font:600 .63rem/1 var(--font-meta); letter-spacing:.15em; text-transform:uppercase; }
  h1,h2,p { margin:0; } h1 { margin-top:.35rem; font-size:clamp(1.7rem,3.2vw,3rem); letter-spacing:-.055em; } h2 { font-size:1.05rem; letter-spacing:-.025em; } .lab-intro { margin-top:.45rem; color:#9eafad; font-size:.82rem; }
  .window-material-status,.shell-foot { display:flex; align-items:center; gap:.65rem; color:#d7e6e3; } .window-material-status small,.shell-foot small { display:block; margin-top:.12rem; color:#7d918f; font-size:.66rem; } .status-filament,.tiny-dot { display:block; width:7px; height:7px; border-radius:50%; background:#79e8df; box-shadow:0 0 0 4px rgba(121,232,223,.08),0 0 18px rgba(121,232,223,.4); }
  .lab-grid { display:grid; grid-template-columns:190px minmax(0,1fr) 250px; gap:12px; align-items:start; }
  .material-shell,.glass-surface { position:relative; overflow:hidden; border:1px solid rgba(221,247,244,.14); background:rgba(9,15,17,.34); box-shadow:inset 0 1px 0 rgba(239,255,252,.13),inset 0 -1px 0 rgba(0,0,0,.82),0 20px 50px rgba(0,0,0,.28); backdrop-filter:blur(30px) saturate(145%); -webkit-backdrop-filter:blur(30px) saturate(145%); }
  .material-shell::before,.glass-surface::before { content:''; position:absolute; z-index:0; inset:0; pointer-events:none; background:linear-gradient(112deg,rgba(255,255,255,.11) 0%,rgba(204,255,251,.025) 16%,transparent 33%,transparent 71%,rgba(132,225,220,.035) 100%); clip-path:polygon(0 0,100% 0,100% 1px,0 1px); transition:opacity .22s ease,transform .22s ease; }
  .material-shell::after,.glass-surface::after { content:''; position:absolute; z-index:0; inset:1px; pointer-events:none; border-radius:inherit; box-shadow:inset 0 0 32px rgba(0,0,0,.32),inset 0 -22px 45px rgba(0,0,0,.16); }
  .material-shell > *,.glass-surface > * { position:relative; z-index:1; }
  .material-shell { min-height:630px; padding:1.05rem .75rem; border-radius:18px; background:rgba(5,10,12,.38); }
  .shell-mark { display:flex; align-items:center; gap:.6rem; padding:.15rem .3rem 1.6rem; } .mark-core { display:grid; place-items:center; width:30px; height:30px; border:1px solid rgba(151,246,237,.58); border-radius:9px; color:#d9fffb; background:linear-gradient(145deg,rgba(137,240,231,.17),rgba(8,20,22,.2)); box-shadow:inset 0 1px 0 rgba(255,255,255,.18),0 0 20px rgba(62,224,215,.12); font-weight:750; } .shell-mark strong,.shell-foot strong { display:block; font-size:.72rem; letter-spacing:.12em; } .shell-mark small { display:block; margin-top:.12rem; color:#718886; font:600 .5rem/1 var(--font-meta); letter-spacing:.1em; }
  .section-label { margin:.2rem .45rem .55rem; color:#5f7674; } .material-nav { display:grid; gap:3px; } .material-nav a { position:relative; display:flex; align-items:center; gap:.65rem; min-height:42px; padding:.55rem .55rem; border:1px solid transparent; border-radius:10px; color:#8ea19f; font-size:.74rem; text-decoration:none; transition:color .18s ease,background .18s ease,border-color .18s ease; } .material-nav a:hover,.material-nav a.selected { color:#e9f8f5; background:rgba(128,239,232,.085); border-color:rgba(146,244,237,.18); } .material-nav a.selected { box-shadow:inset 1px 0 0 rgba(205,255,251,.8),inset 0 1px 0 rgba(225,255,252,.12); } .nav-glyph { width:18px; color:#6e8e8a; text-align:center; } .selected .nav-glyph { color:#a8fff6; } .nav-filament { position:absolute; top:8px; bottom:8px; left:-1px; width:1px; background:linear-gradient(180deg,transparent,#c6fffa,transparent); opacity:0; } .selected .nav-filament { opacity:1; }
  .shell-foot { position:absolute; right:.75rem; bottom:1rem; left:.75rem; padding:.7rem .55rem; border-top:1px solid rgba(220,250,246,.1); } .shell-foot strong { color:#a8bfbc; font:600 .63rem/1 var(--font-body); letter-spacing:0; } .shell-foot small { font-size:.61rem; }
  .lab-workspace { min-width:0; display:grid; gap:12px; } .workspace-bar { min-height:44px; padding:.15rem .3rem .55rem; border-bottom:1px solid rgba(223,249,245,.1); } .workspace-bar strong { display:block; margin-top:.22rem; font-size:.84rem; font-weight:600; } .bar-actions { display:flex; align-items:center; gap:.45rem; }
  .button-primary,.button-secondary,.icon-button,.palette-commands button { font:600 .74rem/1 var(--font-body); cursor:pointer; } .button-primary,.button-secondary { position:relative; min-height:38px; padding:.62rem .82rem; border-radius:9px; color:#e8fffb; } .button-primary { display:flex; align-items:center; gap:.7rem; overflow:hidden; border:1px solid rgba(171,255,247,.48); background:linear-gradient(145deg,rgba(83,210,201,.2),rgba(5,18,20,.66) 58%,rgba(0,0,0,.72)); box-shadow:inset 0 1px 0 rgba(239,255,253,.24),inset 0 -8px 18px rgba(0,0,0,.2),0 12px 28px rgba(0,0,0,.3); transition:transform .16s ease,border-color .16s ease,box-shadow .16s ease; } .button-primary:hover,.button-primary:focus-visible { border-color:#aafff6; box-shadow:inset 0 1px 0 rgba(239,255,253,.33),inset 0 -8px 18px rgba(0,0,0,.14),0 0 24px rgba(92,235,224,.13),0 12px 28px rgba(0,0,0,.3); } .button-primary:active,.button-primary.pressed { transform:translateY(1px) scale(.985); box-shadow:inset 0 5px 15px rgba(0,0,0,.3); } .button-light { position:absolute; inset:-30% 38% 35% -12%; transform:rotate(-18deg); background:linear-gradient(90deg,transparent,rgba(211,255,250,.22),transparent); pointer-events:none; } .button-arrow { color:#a9fff7; font-size:1rem; } .button-secondary { border:1px solid rgba(220,247,244,.17); background:rgba(8,13,15,.32); color:#b9c9c6; box-shadow:inset 0 1px 0 rgba(255,255,255,.1),inset 0 -1px 0 rgba(0,0,0,.6); transition:background .16s ease,border-color .16s ease,color .16s ease; } .button-secondary:hover,.button-secondary:focus-visible { color:#effffb; border-color:rgba(190,249,242,.38); background:rgba(125,224,216,.09); } .button-secondary.large { min-height:42px; padding-inline:1rem; } .icon-button { width:34px; height:34px; padding:0; border:1px solid rgba(224,248,245,.13); border-radius:8px; color:#9db2af; background:rgba(7,12,14,.35); }
  .hero-grid { display:grid; grid-template-columns:minmax(0,1.35fr) minmax(260px,.65fr); gap:12px; } .glass-surface { border-radius:16px; } .glass-surface:hover { border-color:rgba(190,249,243,.25); } .glass-surface:hover::before { opacity:.78; transform:translateX(3%); }
  .surface-label { padding:.8rem .9rem; color:#779894; } .surface-state { color:#52706d; font-size:.54rem; } .lens-surface { min-height:324px; } .lens-body { display:grid; grid-template-columns:minmax(170px,.8fr) minmax(0,1.2fr); align-items:center; gap:1rem; padding:1rem 1.25rem 1.5rem; } .lens-instrument { position:relative; display:grid; place-items:center; width:min(230px,100%); aspect-ratio:1; margin:auto; border-radius:50%; background:radial-gradient(circle at 37% 28%,rgba(202,255,249,.09),transparent 24%),radial-gradient(circle,rgba(6,18,20,.4),rgba(0,0,0,.82) 72%); box-shadow:inset 0 2px 8px rgba(237,255,252,.13),inset 0 -18px 28px rgba(0,0,0,.75),0 20px 34px rgba(0,0,0,.45); } .lens-ring { position:absolute; border:1px solid rgba(190,246,241,.16); border-radius:50%; } .ring-outer { inset:3%; border-top-color:rgba(196,255,249,.65); border-right-color:rgba(90,205,199,.24); transform:rotate(-24deg); } .ring-middle { inset:12%; border-color:rgba(111,224,216,.17); border-left-color:rgba(232,255,252,.4); } .ring-inner { inset:27%; border-color:rgba(224,255,251,.1); box-shadow:inset 0 0 20px rgba(78,223,214,.08); } .lens-ticks { position:absolute; inset:7%; display:flex; align-items:flex-start; justify-content:center; } .lens-ticks i { position:absolute; width:1px; height:8px; background:rgba(205,252,247,.46); transform-origin:50% 98px; } .lens-ticks i:nth-child(1){transform:rotate(-60deg)} .lens-ticks i:nth-child(2){transform:rotate(-40deg)} .lens-ticks i:nth-child(3){transform:rotate(-20deg)} .lens-ticks i:nth-child(4){transform:rotate(0)} .lens-ticks i:nth-child(5){transform:rotate(20deg)} .lens-ticks i:nth-child(6){transform:rotate(40deg)} .lens-ticks i:nth-child(7){transform:rotate(60deg)} .lens-ticks i:nth-child(8){transform:rotate(80deg)} .lens-reading { display:grid; place-items:center; gap:.25rem; } .reading-value { color:#eafffa; font:300 4.6rem/.8 var(--font-display); letter-spacing:-.12em; } .reading-unit { color:#75aaa5; font:600 .52rem/1 var(--font-meta); letter-spacing:.16em; } .lens-needle { position:absolute; width:40%; height:1px; transform:rotate(-35deg) translateX(10%); transform-origin:100% 50%; background:linear-gradient(90deg,transparent,#98f7ef); box-shadow:0 0 10px rgba(110,239,226,.4); }
  .lens-copy { padding-right:.4rem; } .lens-copy h2 { margin:.45rem 0 .65rem; font-size:clamp(1.25rem,2vw,1.8rem); letter-spacing:-.05em; } .lens-copy > p:not(.lab-kicker) { color:#9fb3b0; font-size:.78rem; line-height:1.55; } .lens-legend { display:flex; flex-wrap:wrap; gap:.8rem; margin-top:1.25rem; color:#7f9692; font:600 .61rem/1 var(--font-meta); text-transform:uppercase; letter-spacing:.08em; } .lens-legend span { display:flex; align-items:center; gap:.35rem; } .lens-legend i { width:6px; height:6px; border-radius:50%; } .legend-safe { background:#78d9a6; }.legend-review { background:#e6b46a; }.legend-critical { background:#f27878; }
  .evidence-surface { min-height:324px; background:rgba(7,12,14,.64); } .evidence-list { display:grid; gap:0; margin:.35rem .9rem 1rem; } .evidence-list > div { display:grid; grid-template-columns:32px 1fr; gap:.7rem; padding:1rem .25rem; border-top:1px solid rgba(222,249,245,.1); } .evidence-index { color:#6eaaa4; font:600 .62rem/1 var(--font-meta); } .evidence-list strong { display:block; color:#dcebe8; font-size:.78rem; } .evidence-list small { display:block; margin-top:.3rem; color:#819794; font-size:.69rem; line-height:1.4; }
  .action-surface { padding-bottom:1.25rem; } .control-row { justify-content:flex-start; align-items:flex-end; padding:1rem 1.25rem 0; } .control-caption { margin-bottom:.48rem; color:#63827e; font-size:.54rem; } .crystal-palette { padding:1rem; border-color:rgba(222,255,250,.24); background:rgba(10,19,21,.25); box-shadow:inset 0 1px 0 rgba(255,255,255,.2),inset 0 -1px 0 rgba(0,0,0,.88),0 30px 70px rgba(0,0,0,.5); } .palette-top h2 { margin-top:.35rem; font-size:1.25rem; } kbd { padding:.23rem .36rem; border:1px solid rgba(225,251,247,.15); border-radius:5px; color:#8fa8a4; background:rgba(0,0,0,.28); font:600 .6rem/1 var(--font-meta); } .palette-search { display:flex; align-items:center; gap:.55rem; margin-top:1rem; padding:.7rem .75rem; border:1px solid rgba(215,249,244,.16); border-radius:9px; color:#8ca6a2; background:rgba(0,0,0,.25); font-size:.72rem; } .palette-search span:first-child { color:#b2fff8; font-size:1.1rem; } .palette-search kbd { margin-left:auto; } .palette-commands { display:grid; gap:3px; margin-top:.6rem; } .palette-commands button { display:grid; grid-template-columns:26px 1fr auto; align-items:center; gap:.55rem; padding:.65rem .55rem; border:1px solid transparent; border-radius:8px; color:#9db2af; background:transparent; text-align:left; } .palette-commands button:hover,.palette-commands button:focus-visible { border-color:rgba(190,249,243,.2); color:#e8f9f6; background:rgba(119,226,217,.08); } .command-icon { color:#91eee6; font-size:1rem; }.palette-commands strong,.palette-commands small { display:block; }.palette-commands strong { font-size:.73rem; }.palette-commands small { margin-top:.2rem; color:#748b87; font-size:.64rem; }
  .crystal-inspector { min-height:630px; padding:1rem; background:rgba(13,23,25,.23); border-color:rgba(223,252,248,.2); } .inspector-head { align-items:flex-start; } .inspector-head h2 { margin-top:.35rem; font-size:1.18rem; }.inspector-preview { display:grid; align-content:end; min-height:190px; margin:1.2rem 0; padding:.8rem; overflow:hidden; border:1px solid rgba(207,251,246,.18); border-radius:12px; background:linear-gradient(145deg,rgba(205,255,251,.1),rgba(10,18,20,.12) 38%,rgba(0,0,0,.58)); box-shadow:inset 0 1px 0 rgba(255,255,255,.2),inset 0 -22px 30px rgba(0,0,0,.4); } .preview-glint { position:absolute; width:130%; height:28px; margin:-6rem 0 0 -1rem; transform:rotate(-20deg); background:linear-gradient(90deg,transparent,rgba(227,255,251,.18),transparent); } .inspector-preview strong { font-size:.82rem; }.inspector-preview small { margin-top:.3rem; color:#91aaa6; font-size:.67rem; line-height:1.4; }.inspector-list { display:grid; gap:0; margin:0; }.inspector-list div { display:flex; justify-content:space-between; gap:.6rem; padding:.7rem 0; border-top:1px solid rgba(222,249,245,.1); }.inspector-list dt,.inspector-list dd { font:600 .62rem/1 var(--font-meta); }.inspector-list dt { color:#708b87; }.inspector-list dd { margin:0; color:#c1d3cf; }.inspector-note { margin-top:1.2rem; padding-top:1rem; border-top:1px solid rgba(222,249,245,.1); color:#819894; font-size:.68rem; line-height:1.5; }
  /* The prototype follows the production palette: silver light, green/red semantics. */
  .ambient-cyan,.ambient-indigo { background:radial-gradient(circle,rgba(214,222,225,.11),transparent 66%); }
  .material-nav a:hover,.material-nav a.selected { background:color-mix(in srgb,var(--ui-accent) 10%,transparent); border-color:color-mix(in srgb,var(--ui-accent) 34%,transparent); }
  .selected .nav-glyph,.nav-filament,.button-arrow,.command-icon { color:var(--ui-accent); }
  .nav-filament { background:linear-gradient(180deg,transparent,var(--ui-accent),transparent); }
  .button-primary { border-color:color-mix(in srgb,var(--ui-accent) 70%,white); background:linear-gradient(145deg,color-mix(in srgb,var(--ui-accent) 20%,transparent),rgba(5,10,12,.72) 58%,#000 100%); }
  .button-primary:hover,.button-primary:focus-visible { border-color:var(--ui-accent-hover); box-shadow:inset 0 1px 0 var(--ui-specular),inset 0 -8px 18px rgba(0,0,0,.14),0 0 24px color-mix(in srgb,var(--ui-accent) 16%,transparent),0 12px 28px rgba(0,0,0,.3); }
  .button-light { background:linear-gradient(90deg,transparent,var(--ui-specular),transparent); }
  .ring-outer { border-top-color:var(--ui-accent); border-right-color:color-mix(in srgb,var(--ui-accent) 28%,transparent); }
  .ring-middle { border-color:color-mix(in srgb,var(--ui-accent) 22%,transparent); border-left-color:color-mix(in srgb,var(--ui-accent) 66%,transparent); }
  .lens-ticks i { background:color-mix(in srgb,var(--ui-accent) 58%,transparent); }
  .lens-needle { background:linear-gradient(90deg,transparent,var(--ui-accent)); box-shadow:0 0 10px color-mix(in srgb,var(--ui-accent) 34%,transparent); }
  .reading-value,.button-primary,.button-secondary:hover,.button-secondary:focus-visible { color:var(--ui-text-primary); }
  .legend-review { background:var(--ui-warning); }
  @media (max-width:1180px) { .lab-grid { grid-template-columns:170px minmax(0,1fr); }.crystal-inspector { grid-column:2; min-height:0; }.inspector-list { grid-template-columns:1fr 1fr; display:grid; column-gap:1rem; } }
  @media (max-width:760px) { .material-lab { padding:.5rem; }.lab-frame { padding:.75rem; border-radius:16px; }.lab-header { align-items:flex-start; flex-direction:column; }.lab-grid,.hero-grid { grid-template-columns:1fr; }.material-shell,.crystal-inspector { grid-column:auto; min-height:0; }.material-shell { padding-bottom:4.7rem; }.material-nav { grid-template-columns:repeat(3,1fr); }.material-nav a { justify-content:center; flex-direction:column; gap:.25rem; text-align:center; font-size:.63rem; }.shell-foot { bottom:.7rem; }.lens-body { grid-template-columns:1fr; }.lens-copy { text-align:center; }.lens-legend { justify-content:center; }.control-row { align-items:stretch; flex-direction:column; }.button-primary,.button-secondary.large { width:100%; justify-content:center; }.window-material-status { align-self:stretch; }.workspace-bar { align-items:flex-start; flex-direction:column; } }
  @media (prefers-reduced-transparency: reduce), (forced-colors: active) { .material-lab { background:#020506; }.material-shell,.glass-surface,.crystal-palette,.crystal-inspector { background:#0b1214; backdrop-filter:none; -webkit-backdrop-filter:none; box-shadow:inset 0 1px 0 rgba(239,255,252,.16),inset 0 -1px 0 rgba(0,0,0,.9); }.ambient { display:none; } }
  @media (prefers-reduced-motion: reduce) { .material-shell::before,.glass-surface::before,.button-primary,.button-secondary,.material-nav a { transition:none; } }
</style>
