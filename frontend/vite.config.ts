import { defineConfig, loadEnv } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import unocssPlugin from "unocss/vite"
import shell from "shelljs"

export default defineConfig(({ mode }) => {
  return {
    plugins: [solidPlugin(), unocssPlugin(), {
      name: 'bundle_xdc',
      closeBundle: () => {
        //@ts-ignore
        const env = loadEnv(mode, process.cwd(), '')
        if (env.VITE_APPSTORE) {
          shell.exec('./create_xdc.sh appstore')
        } else {
          shell.exec('./create_xdc.sh review_helper')
        }
      }
    }],
    server: {
      port: 3000,
    }
  }
});
