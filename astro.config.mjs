import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import remarkHeadingId from "remark-heading-id";

export default defineConfig({
  site: "https://ofcr.se/",
  integrations: [sitemap()],
  scopedStyleStrategy: "class",
  markdown: {
    shikiConfig: {
      theme: 'github-dark-dimmed',
      wrap: false,
    },
    remarkPlugins: [remarkHeadingId],
  },
});
