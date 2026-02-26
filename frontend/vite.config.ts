import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		// During development, proxy API calls to the Rust backend.
		proxy: {
			'/api': 'http://localhost:3000'
		}
	}
});
