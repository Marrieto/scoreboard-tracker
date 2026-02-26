// Disable server-side rendering — we're building a static SPA.
// All routing happens client-side; the Rust backend only serves the API.
// We use the fallback page (index.html) for all routes, so prerender
// is false — dynamic routes like /players/[id] can't be known at build time.
export const ssr = false;
export const prerender = false;
