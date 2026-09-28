import {URLMatcher, deLocalizeDefault} from "wuchale/url"
import {locales} from "./data.js"
import manifest from "./.wuchale/main.urls.js"
export const getLocale = (/** @type {URL} */ url) => deLocalizeDefault(url.pathname, locales)[1] ?? 'eng'
export const matchUrl = URLMatcher(manifest, locales)