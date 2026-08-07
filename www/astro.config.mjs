import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://cmb.software",
  base: "/my-prompt",
  output: "static",
  outDir: "./dist/my-prompt",
  trailingSlash: "always",
});
