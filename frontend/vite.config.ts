import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { execSync } from 'node:child_process'
import { defineConfig, loadEnv } from 'vite'
import solidPlugin from 'vite-plugin-solid'
import unocssPlugin from 'unocss/vite'

function eruda() {
  const erudaSrc = readFileSync("./node_modules/eruda/eruda.js", "utf-8");
  return {
    name: "vite-plugin-eruda",
    transformIndexHtml(html) {
      const tags = [
        {
          tag: "script",
          children: erudaSrc,
          injectTo: "head",
        },
        {
          tag: "script",
          children: "eruda.init();",
          injectTo: "head",
        },
      ];
      // @ts-ignore
      if (process.env.NODE_ENV !== "production") {
        return {
          html,
          tags,
        };
      } else {
        return html;
      }
    },
  };
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const app_name = env.VITE_APP
  process.env.VITE_COMMIT = String(execSync('git describe --always'))

  return {
    plugins: [eruda(), solidPlugin(), unocssPlugin()],
    build: {
      target: ['es2020', 'edge88', 'firefox78', 'chrome74', 'safari14'],
      rollupOptions: {
        input: {
          shop: resolve(__dirname, `./${app_name}.html`),
        },
        output: {
          dir: `dist/${app_name}`,
        },
      },
    },
    server: {
      port: 3000,
    },
    test: {
      environment: 'jsdom',
      transformMode: {
        web: [/.[jt]sx?/],
      },
      threads: false,
      isolate: false,
    },
  }
})
