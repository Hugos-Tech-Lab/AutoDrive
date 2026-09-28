import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { loadLocale } from 'wuchale/load-utils';

import type { Locale } from '$lib/Locale';

import '../locales/main.loader.svelte.js';
import '../locales/js.loader.js';

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

		window.location.replace(
			`/${locale}${window.location.pathname}${window.location.search}${window.location.hash}`
		);
	}

	await loadLocale(locale);

	return { locale };
};

export const prerender = true;
