import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			assets: '../static',
			fallback: 'index.html',
			pages: '../static'
		})
	}
};

export default config;
