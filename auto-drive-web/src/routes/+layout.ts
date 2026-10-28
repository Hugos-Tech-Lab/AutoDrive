import { redirect } from '@sveltejs/kit';
import { loadLocale } from 'wuchale/load-utils';
import type { Locale } from '$lib/Locale';

import '../locales/main.loader.svelte.js';
import '../locales/js.loader.js';

export const ssr = false;
export const prerender = false;

export const load = async ({ url }) => {
  const match = url.pathname.match(/^\/(eng|cze|ukr)(?:\/|$)/);

  if (match) {
    const locale = match[1] as Locale;
    await loadLocale(locale);
    return { locale };
  }

  // Detect locale from browser or default to eng
  let targetLocale: Locale = 'eng';

  if (typeof navigator !== 'undefined') {
    const language = navigator.language.toLowerCase();
    if (language.startsWith('cs')) {
      targetLocale = 'cze';
    } else if (language.startsWith('uk')) {
      targetLocale = 'ukr';
    }
  }

  const targetPath = `/${targetLocale}${url.pathname}${url.search}`;
  
  // Prevent infinite loops if target path is already root
  if (`/${targetLocale}` !== url.pathname) {
    throw redirect(307, targetPath);
  }

  await loadLocale(targetLocale);
  return { locale: targetLocale };
};