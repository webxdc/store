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
      button: '#e5eaff',
      buttonLight: '#d2d9f5',
    },
    breakpoints: {
      xs: '380px',
      sm: '640px',
      md: '768px',
      lg: '1024px',
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
    btn: 'rounded-md p-1 bg-gray-200 hover:bg-gray-300 tracking-wide text-gray-500',
    unimportant: 'text-gray-400 font-italic tracking-wide',
  },
})
