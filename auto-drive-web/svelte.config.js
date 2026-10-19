import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({
      pages: "dist",
      assets: "dist",
      fallback: "404.html",
      precompress: false,
      strict: true,
    }),
    prerender: {
      entries: ["/", "/eng", "/cze", "/ukr"],
      handleUnseenRoutes: "ignore",
    },
    paths: {
      relative: false,
    },
  },
};

export default config;
