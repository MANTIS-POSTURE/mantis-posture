<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	onMount(() => { goto('/posture', { replaceState:true }); });
</script>

<div class="launch" role="status" aria-label="Ouverture de MANTIS POSTURE">
	<div class="radar" aria-hidden="true"><span class="sweep"></span><i></i><i></i><i></i><strong>M</strong></div>
	<div class="launch-copy"><span>MANTIS POSTURE</span><p>Initialisation du centre de renseignement local</p></div>
	<div class="launch-progress" aria-hidden="true"><span></span></div>
</div>

<style>
	.launch{position:relative;display:grid;place-items:center;align-content:center;gap:1.25rem;width:100%;height:100%;min-height:calc(100vh - var(--titlebar-height));overflow:hidden;background:radial-gradient(circle at center,rgba(214,222,225,.06),transparent 32%)}
	.radar{position:relative;display:grid;place-items:center;width:116px;height:116px;border:1px solid rgba(214,222,225,.24);border-radius:50%;background:radial-gradient(circle,rgba(214,222,225,.1),rgba(0,0,0,.18) 68%);box-shadow:inset 0 0 34px rgba(214,222,225,.07),0 0 45px rgba(214,222,225,.08);overflow:hidden}.radar::before,.radar::after,.radar i{content:'';position:absolute;border:1px solid rgba(214,222,225,.12);border-radius:50%}.radar::before{inset:22px}.radar::after{inset:42px}.radar i{width:1px;height:100%;border:0;border-left:1px solid rgba(214,222,225,.1);border-radius:0}.radar i:nth-of-type(2){transform:rotate(60deg)}.radar i:nth-of-type(3){transform:rotate(120deg)}.radar strong{z-index:2;color:var(--ui-accent);font-size:1.4rem}.sweep{position:absolute;inset:50% 0 0 50%;transform-origin:0 0;background:linear-gradient(35deg,rgba(214,222,225,.28),transparent 58%);animation:sweep 1.8s linear infinite}.launch-copy{text-align:center}.launch-copy span{font-size:.78rem;font-weight:750;letter-spacing:.18em}.launch-copy p{margin:.35rem 0 0;color:var(--ui-text-secondary);font-size:.75rem}.launch-progress{width:210px;height:2px;overflow:hidden;border-radius:99px;background:rgba(255,255,255,.07)}.launch-progress span{display:block;width:42%;height:100%;background:var(--ui-accent);box-shadow:0 0 10px var(--ui-accent);animation:loading 1.3s ease-in-out infinite}@keyframes sweep{to{transform:rotate(360deg)}}@keyframes loading{0%{transform:translateX(-110%)}100%{transform:translateX(350%)}}@media(prefers-reduced-motion:reduce){.sweep,.launch-progress span{animation:none}.launch-progress span{width:100%;opacity:.5}}
</style>
