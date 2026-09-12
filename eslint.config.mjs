import antfu from '@antfu/eslint-config'
import vueI18n from '@intlify/eslint-plugin-vue-i18n'

export default antfu(
  {
    vue: true,
    typescript: true,
    formatters: true,
    gitignore: true,
    ignores: [
      '**/components/ui/**',
      '**/src-tauri/**',
      '**/vite-env.d.ts',
      'vite.config.ts',
      'tsconfig.json',
      'tsconfig.app.json',
      'tsconfig.node.json',
      '.github/**',
      'docs/**',
    ],
    stylistic: {
      indent: 2,
      quotes: 'single',
    },
  },
  ...vueI18n.configs.base,
  {
    files: ['**/*.vue'],
    settings: {
      'vue-i18n': {
        localeDir: './src/i18n/locales/*.json',
        messageSyntaxVersion: '^11.0.0',
      },
    },
  },
)
