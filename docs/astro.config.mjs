// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://jake-walker.github.io",
  base: "rs-arcade-coder",
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
