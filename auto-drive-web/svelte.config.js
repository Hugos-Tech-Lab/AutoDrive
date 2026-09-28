import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({
			// default options are shown. On some platforms
			// these options are set automatically — see below
			pages: 'dist',
			assets: 'dist',
			fallback: '404.html',
			precompress: false,
			strict: true
		}),
		prerender: {
			entries: [,
				'/',
				'/eng',
				'/cze',
				'/ukr'
			],
			handleUnseenRoutes: 'ignore'
		},
	}

};

export default config;
