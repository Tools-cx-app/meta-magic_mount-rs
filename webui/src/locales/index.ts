// src/locales/index.ts
import { createI18n } from 'vue-i18n'

// 动态导入同目录下的所有 JSON 文件
const localeModules = import.meta.glob('./*.json', { eager: false })

// 缓存语言列表
let cachedLocales: { code: string; display: string }[] | null = null

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: {},
})

// 获取支持的语言列表（从 JSON 文件读取 lang.display）
export async function getSupportedLocales() {
  if (cachedLocales) return cachedLocales
  
  const results = await Promise.all(
    Object.entries(localeModules).map(async ([path, loader]) => {
      const match = path.match(/\.\/(.+)\.json$/)
      if (!match) return null
      
      const code = match[1]
      const mod = await loader() as { default: Record<string, unknown> }
      const messages = mod.default as { lang?: { display?: string } }
      
      return {
        code,
        display: messages.lang?.display || code,
      }
    })
  )
  
  cachedLocales = results.filter(Boolean) as { code: string; display: string }[]
  return cachedLocales
}

export async function getCurrentLangIndex(): Promise<number> {
  const locales = await getSupportedLocales()
  const currentLocale = i18n.global.locale.value
  return locales.findIndex(lang => lang.code === currentLocale)
}

// 加载单个语言包
export async function loadLocale(locale: string) {
  if (i18n.global.availableLocales.includes(locale)) return
  
  const path = `./${locale}.json`
  const loader = localeModules[path]
  if (!loader) {
    console.error(`Locale "${locale}" not found`)
    return
  }
  
  const module = (await loader()) as { default: Record<string, unknown> }
  i18n.global.setLocaleMessage(locale, module.default)
}

// 切换语言
export async function switchLocale(locale: string) {
  await loadLocale(locale)
  i18n.global.locale.value = locale
  localStorage.setItem('locale', locale)
}

// 初始化 i18n
export async function initI18n(preferred?: string) {
  const locales = await getSupportedLocales()
  if (locales.length === 0) {
    console.error('No locale files found!')
    return
  }
  
  const savedLocale = localStorage.getItem('locale')
  let defaultLocale = preferred || savedLocale || locales[0].code
  
  if (!locales.some(l => l.code === defaultLocale)) {
    defaultLocale = locales[0].code
  }
  
  await loadLocale(defaultLocale)
  i18n.global.locale.value = defaultLocale
}

export default i18n