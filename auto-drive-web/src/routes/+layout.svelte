<script lang="ts">
  import "../app.css";
  import cousinFont from "../assets/fonts/Cousin-Bold.woff2";

  let { children } = $props();
</script>

<svelte:head>
  <link
    rel="preload"
    as="font"
    href={cousinFont}
    type="font/woff2"
    crossorigin="anonymous"
  />
</svelte:head>

{@render children()}

<style lang="postcss">
  @reference "tailwindcss";
  :global(html) {
    font-family: "Cousin", system-ui, sans-serif;
    color: #08060d;
    background: rgb(245, 249, 255);
    font-weight: bold;
  }

	:global(body) {
		margin: 0;
	}

	:global(section) {
		min-height: 100vh;
		box-sizing: border-box;
		border-bottom: 10px solid rgb(0, 0, 0);
		position: relative;
	}

	:global(.main) {
    border: 1px solid rgb(0, 36, 156);
	}

	:global(form) {
    border: 1px solid rgb(0, 36, 156);
	}

  :global(.section-inner) {
    margin: 0 auto;
    /* padding: 2vw; */
    max-width: 60rem;
  }
</style>
