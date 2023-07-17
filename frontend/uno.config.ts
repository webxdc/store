import {
  defineConfig,
  extractorSplit,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetUno,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss'

export default defineConfig({
  theme: {
    colors: {
      button: '#e5eaff',
      buttonLight: '#d2d9f5',
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
