export interface Language {
  value: string
  label: string
}

export function supportedLanguages(): Language[] {
  return [
    {
      label: 'English',
      value: 'en',
    },
    {
      label: '中文',
      value: 'zh',
    },
    {
      label: 'Español',
      value: 'es',
    },
  ]
}

export function getLanguageLabel(value: string): string {
  return supportedLanguages().find(lang => lang.value === value)?.label ?? value
}

/** 把应用 i18n locale 映射成 Intl / Calendar 可用的 BCP 47 标签 */
export function toIntlLocale(appLocale: string): string {
  switch (appLocale) {
    case 'zh':
      return 'zh-CN'
    case 'es':
      return 'es'
    case 'en':
    default:
      return 'en-US'
  }
}
