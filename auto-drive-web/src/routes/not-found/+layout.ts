import { browser } from '$app/environment';
import { loadLocale } from 'wuchale/load-utils';
import type { Locale } from '$lib/Locale';

import '../../locales/main.loader.svelte.js';
import './../../locales/js.loader.js';
import './../../app.css';

// This is the root so it's strange to track url here,
// but it's necessary so SvelteKit reruns this (and most
// importantly, reruns loadLocale) when going to /.
export const load = async ({ url }) => {
	let locale: Locale = 'eng';

	const match = url.pathname.match(/^\/(eng|cze|ukr)(?:\/|$)/);

	if (match) {
		locale = match[1] as Locale;
	} else if (browser) {
		const language = navigator.language.toLowerCase();

		if (language.startsWith('cs')) {
			locale = 'cze';
		} else if (language.startsWith('uk')) {
			locale = 'ukr';
		}
	}

	await loadLocale(locale);
	return { locale };
};

export const prerender = true;
