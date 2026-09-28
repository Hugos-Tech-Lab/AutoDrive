<script lang="ts">
  import type { Locale } from "$lib/Locale";
  import { loadLocale } from "wuchale/load-utils";
  import { goto } from "$app/navigation";

  let { locale }: { locale: Locale } = $props();

  async function changeLocale(newLocale: Locale) {
    if (newLocale === locale) return;

    await loadLocale(newLocale);

    const pathWithoutLocale = window.location.pathname.replace(
      /^\/(eng|cze|ukr)(?=\/|$)/,
      "",
    );

    const newPath = `/${newLocale}${pathWithoutLocale}`;

    await goto(newPath);
  }
</script>

<div>
  <button class:active={locale === "eng"} onclick={() => changeLocale("eng")}>ENG</button>
  <button class:active={locale === "cze"} onclick={() => changeLocale("cze")}>CZE</button>
  <button class:active={locale === "ukr"} onclick={() => changeLocale("ukr")}>UKR</button>
</div>

<style>
  button.active {
    opacity: 1;
    background: #252283;
    color: #fff9e0;
  }
</style>
