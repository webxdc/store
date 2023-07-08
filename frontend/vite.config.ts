import path from 'node:path'
import { readFileSync } from 'node:fs'
import { execSync } from 'node:child_process'
import { defineConfig } from 'vite'
import solidPlugin from 'vite-plugin-solid'
import unocssPlugin from 'unocss/vite'

function eruda() {
  const erudaSrc = readFileSync('./node_modules/eruda/eruda.js', 'utf-8')
  return {
    name: 'vite-plugin-eruda',
    transformIndexHtml(html: any) {
      const tags = [
        {
          tag: 'script',
          children: erudaSrc,
          injectTo: 'head',
        },
        {
          tag: 'script',
          children: 'eruda.init();',
          injectTo: 'head',
        },
      ]
      if (process.env.ERUDA) {
        return {
          html,
          tags,
        }
      }
      else {
        return html
      }
    },
  }
}

export default defineConfig(() => {
  process.env.VITE_COMMIT = String(execSync('git describe --always').toString().trim())
  return {
    resolve: {
      alias: {
        '~/': `${path.resolve(__dirname, 'src')}/`,
      },
    },
    plugins: [eruda(), solidPlugin(), unocssPlugin()],
    build: {
      target: ['es2020', 'edge88', 'firefox78', 'chrome74', 'safari14'],
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
