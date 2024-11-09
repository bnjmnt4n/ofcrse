import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import rehypePrettyCode from "rehype-pretty-code";
import remarkHeadingId from "remark-heading-id";

export default defineConfig({
  site: "https://ofcr.se/",
  integrations: [sitemap()],
  scopedStyleStrategy: "class",
  markdown: {
    syntaxHighlight: false,
    rehypePlugins: [
      [
        rehypePrettyCode,
        {
          theme: "github-dark-dimmed",
          keepBackground: true,
          defaultLang: {
            block: "plaintext",
          },
        },
      ],
    ],
    remarkPlugins: [remarkHeadingId],
  },
});
