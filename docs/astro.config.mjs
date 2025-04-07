// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import astroRehypeRelativeMarkdownLinks from "astro-rehype-relative-markdown-links";

// https://astro.build/config
export default defineConfig({
  site: "https://jake-walker.github.io",
  base: "rs-arcade-coder",
  markdown: {
    rehypePlugins: [
      [
        astroRehypeRelativeMarkdownLinks,
        {
          base: "/rs-arcade-coder",
          collectionBase: false,
        },
      ],
    ],
  },
  integrations: [
    starlight({
      title: "Arcade Coder Wiki",
      social: {
        github: "https://github.com/jake-walker/arcade-coder-wiki",
      },
      sidebar: [
        {
          label: "Hardware",
          items: ["hardware/overview", "hardware/display", "hardware/buttons"],
        },
        {
          label: "Software & Programming",
          autogenerate: { directory: "software" },
        },
        {
          label: "Links",
          link: "links",
        },
      ],
    }),
  ],
});
