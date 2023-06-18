import {
  defineConfig,
  extractorSplit,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetUno,
  presetWebFonts,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'

export default defineConfig({
  theme: {
    colors: {
      buttonBg: '#e5eaff',
    },
  },
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
      warn: true,
    }),
    presetTypography(),
    presetWebFonts({
      fonts: {
        sans: 'DM Sans',
        serif: 'DM Serif Display',
        mono: 'DM Mono',
      },
    }),
  ],
  extractors: [
    extractorSplit,
  ],
  transformers: [
    transformerDirectives(),
    transformerVariantGroup(),
  ],
  shortcuts: {
    btn: 'rounded border p-1 bg-buttonBg hover:bg-gray-200 text-blue-800',
    unimportant: 'text-gray-400 font-italic tracking-wide',
  },
})
