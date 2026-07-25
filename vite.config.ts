import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// MANTIS est une app desktop Tauri : le frontend est buildé en assets
			// statiques (adapter-static) et embarqué dans le binaire, sans serveur Node.
			adapter: adapter()
		})
	],

	// Le dev server doit toujours utiliser le port 5173 : c'est celui que Tauri
	// attend via build.devUrl dans src-tauri/tauri.conf.json.
	server: {
		port: 5173,
		strictPort: true,
		watch: {
			// Ne jamais surveiller le code Rust : pendant la compilation, Cargo
			// verrouille des fichiers dans src-tauri/target, ce qui fait planter
			// le watcher Vite sous Windows (EBUSY) et tue le dev server.
			ignored: ['**/src-tauri/**']
		}
	}
});
