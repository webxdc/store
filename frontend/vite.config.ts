import { defineConfig, loadEnv } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import unocssPlugin from "unocss/vite"
//@ts-ignore
import { resolve } from 'path'

export default defineConfig(({ mode }) => {
  //@ts-ignore
  const env = loadEnv(mode, process.cwd(), '')
  const app_name = env.VITE_APP || 'shop'

  return {
    plugins: [solidPlugin(), unocssPlugin()],
    build: {
      rollupOptions: {
        input: {
          //@ts-ignore
          shop: resolve(__dirname, `./${app_name}.html`),
        },
        output: {
          dir: 'dist/' + app_name
        }
      },
    },
    server: {
      port: 3000,
    }
  }
});
