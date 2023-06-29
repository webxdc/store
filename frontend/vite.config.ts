import { resolve } from 'node:path'
import { execSync } from 'child_process'
import { defineConfig, loadEnv } from 'vite'
import solidPlugin from 'vite-plugin-solid'
import unocssPlugin from 'unocss/vite'

// @ts-expect-error

export default defineConfig(({ mode }) => {
  // @ts-expect-error
  const env = loadEnv(mode, process.cwd(), '')
  const app_name = env.VITE_APP
  process.env.VITE_COMMIT = String(execSync('git describe --always'))

  return {
    plugins: [solidPlugin(), unocssPlugin()],
    build: {
      target: ['es2020', 'edge88', 'firefox78', 'chrome74', 'safari14'],
      rollupOptions: {
        input: {
          // @ts-expect-error
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
  }
})
