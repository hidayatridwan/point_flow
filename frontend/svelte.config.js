import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	// The app is a client-only SPA (`ssr = false` in the root layout), so we ship a
	// static build with a fallback shell instead of a Netlify serverless function.
	kit: { adapter: adapter({ fallback: '200.html' }) }
};

export default config;
