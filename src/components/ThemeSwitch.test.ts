import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import ThemeSwitch from './ThemeSwitch.vue'

vi.mock('@tauri-apps/plugin-store', () => {
  class MockLazyStore {
    get = vi.fn()
    set = vi.fn()
    save = vi.fn()
    entries = vi.fn().mockResolvedValue([])
  }
  return { LazyStore: MockLazyStore }
})

vi.mock('@vueuse/core', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@vueuse/core')>()
  return {
    ...actual,
    useColorMode: vi.fn(() => ({ value: 'auto' })),
  }
})

const i18n = createI18n({
  locale: 'en',
  legacy: false,
  messages: {
    en: {
      settings: {
        theme: {
          label: 'Theme',
          light: 'Light',
          dark: 'Dark',
          system: 'System',
        },
      },
    },
  },
})

function mountThemeSwitch() {
  return mount(ThemeSwitch, {
    global: {
      plugins: [i18n],
    },
  })
}

describe('themeSwitch', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('renders radio options for light, dark, and system', () => {
    const wrapper = mountThemeSwitch()
    const radios = wrapper.findAll('[data-slot="radio-group-item"]')
    expect(radios).toHaveLength(3)
    expect(wrapper.text()).toContain('Light')
    expect(wrapper.text()).toContain('Dark')
    expect(wrapper.text()).toContain('System')
  })

  it('exposes an accessible theme group label', () => {
    const wrapper = mountThemeSwitch()
    const group = wrapper.find('[data-slot="radio-group"]')
    expect(group.attributes('aria-label')).toBe('Theme')
  })
})
