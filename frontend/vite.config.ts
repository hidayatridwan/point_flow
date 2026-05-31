import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		host: true,
		// polling is required for hot reload inside Docker on macOS
		watch: { usePolling: !!process.env.DOCKER_DEV }
	}
});
