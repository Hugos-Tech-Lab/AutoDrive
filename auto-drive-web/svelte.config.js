import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({
      pages: "dist",
      assets: "dist",
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),
    prerender: {
      handleUnseenRoutes: "ignore",
    },
    paths: {
      relative: false,
    },
  },
};

export default config;