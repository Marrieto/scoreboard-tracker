import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			// Output directory for the built static files.
			// The Rust server serves these from the `static/` directory.
			fallback: 'index.html' // SPA mode: all routes fall back to index.html
		})
	}
};

export default config;
