import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		alias: {
			"$action": "./www-root/action",
			"$cmp": "./www-root/components",
			"$icons": "./www-root/icons",
			"$styles": "./www-root/styles",
			"$assets": "./www-root/assets",
		},
		files: {
			lib: "./www-root",
			routes: "./routes",
			appTemplate: "app.html",
			assets: "./www-root/assets"
		},
		adapter: adapter()
	}
};

export default config;
