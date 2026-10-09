import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

const THEME_KEY = 'of-theme'

type ThemeMode = 'light' | 'dark' | 'system'

function readStoredTheme(): ThemeMode {
  const raw = localStorage.getItem(THEME_KEY)
  return raw === 'light' || raw === 'dark' || raw === 'system' ? raw : 'system'
}

function systemPrefersDark(): boolean {
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false
}

/** 全局 UI 状态：主题、移动端抽屉 */
export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>(readStoredTheme())
  const prefersDark = ref(systemPrefersDark())
  /** 左侧社区抽屉（移动端） */
  const drawerOpen = ref(false)

  const isDark = computed(() =>
    theme.value === 'system' ? prefersDark.value : theme.value === 'dark',
  )

  function apply() {
    document.documentElement.classList.toggle('dark', isDark.value)
    document.documentElement.style.colorScheme = isDark.value ? 'dark' : 'light'
  }

  function setTheme(mode: ThemeMode) {
    theme.value = mode
    localStorage.setItem(THEME_KEY, mode)
    apply()
  }

  function toggleTheme() {
    setTheme(isDark.value ? 'light' : 'dark')
  }

  function toggleDrawer(open?: boolean) {
    drawerOpen.value = open ?? !drawerOpen.value
  }

  /** 应用启动时调用：跟随系统主题变化 */
  function init() {
    window.matchMedia?.('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      prefersDark.value = e.matches
      if (theme.value === 'system') apply()
    })
    apply()
  }

  return {
    theme,
    isDark,
    drawerOpen,
    setTheme,
    toggleTheme,
    toggleDrawer,
    init,
  }
})
