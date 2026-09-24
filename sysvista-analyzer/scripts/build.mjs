import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createRequire } from "node:module";
import { build } from "esbuild";

const require = createRequire(import.meta.url);
const directory = dirname(require.resolve("typescript/lib/typescript.js"));
const libraries = Object.fromEntries(readdirSync(directory)
  .filter(name => /^lib(\..+)?\.d\.ts$/.test(name))
  .sort()
  .map(name => [name, readFileSync(join(directory, name), "utf8")]));

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  platform: "node",
  format: "esm",
  outfile: "dist/analyzer.js",
  banner: { js: "import{createRequire}from'node:module';import{fileURLToPath}from'node:url';const require=createRequire(import.meta.url),__filename=fileURLToPath(import.meta.url),__dirname=fileURLToPath(new URL('.',import.meta.url));" },
  plugins: [{
    name: "embedded-typescript-libs",
    setup(builder) {
      builder.onResolve({ filter: /^embedded-libs$/ }, () => ({ path: "embedded-libs", namespace: "embedded-libs" }));
      builder.onLoad({ filter: /.*/, namespace: "embedded-libs" }, () => ({ loader: "js", contents: `export const LIBRARIES=${JSON.stringify(libraries)};` }));
    },
  }],
});
