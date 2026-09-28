import * as main from './locales/main.loader.server.svelte.js'
import * as js from './locales/js.loader.server.js'
import { runWithLocale, loadLocales } from 'wuchale/load-utils/server'
import { locales } from './locales/data.js'

// Load catalogs at server startup
loadLocales(main.key, main.loadCount, main.loadCatalog, locales)
loadLocales(js.key, js.loadCount, js.loadCatalog, locales)

/** @type {import('@sveltejs/kit').Handle} */
export const handle = async ({ event, resolve }) => {
	const locale = event.params.locale ?? 'eng'
	return await runWithLocale(locale, () => resolve(event))
}
