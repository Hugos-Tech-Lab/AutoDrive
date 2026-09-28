import { error } from '@sveltejs/kit';
import { loadLocale } from 'wuchale/load-utils';

import '../../locales/main.loader.svelte.js';
import '../../locales/js.loader.js';
import { validLocales } from '$lib/Locale.js';

export const prerender = true;

export const load = async ({ params }: { params: { locale: string } }) => {
	const locale = params.locale;

	if (!validLocales.includes(locale)) {
		throw error(404, 'Page not found');
	}

	await loadLocale(locale);

	return {
		locale
	};
};