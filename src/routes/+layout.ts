// Tauri charge une SPA locale. Le fallback statique permet aux routes de détail
// dynamiques (/dossiers/[id], etc.) d'être résolues côté client.
export const prerender = false;
export const ssr = false;
