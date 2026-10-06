import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { authApi, type LoginPayload, type RegisterPayload } from '@/api'
import type { User } from '@/types'
import { safeJsonParse } from '@/utils/format'

const TOKEN_KEY = 'token'
const USER_KEY = 'user'

function readStoredUser(): User | null {
  return safeJsonParse<User>(localStorage.getItem(USER_KEY))
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
  const user = ref<User | null>(readStoredUser())
  const loading = ref(false)

  const isAuthenticated = computed(() => !!token.value)
  const isAdmin = computed(() => user.value?.role === 'ADMIN')

  function persist() {
    if (token.value) localStorage.setItem(TOKEN_KEY, token.value)
    else localStorage.removeItem(TOKEN_KEY)
    if (user.value) localStorage.setItem(USER_KEY, JSON.stringify(user.value))
    else localStorage.removeItem(USER_KEY)
  }

  function setAuth(t: string, u: User) {
    token.value = t
    user.value = u
    persist()
  }

  function setUser(u: User) {
    user.value = u
    persist()
  }

  function logout() {
    token.value = null
    user.value = null
    persist()
  }

  async function fetchMe() {
    if (!token.value) return
    try {
      const me = await authApi.me()
      setUser(me)
    } catch {
      // 401 已由拦截器清理；其它错误保留本地缓存
    }
  }

  async function login(payload: LoginPayload) {
    loading.value = true
    try {
      const res = await authApi.login(payload)
      setAuth(res.token, res.user)
      return res.user
    } finally {
      loading.value = false
    }
  }

  async function register(payload: RegisterPayload) {
    loading.value = true
    try {
      const res = await authApi.register(payload)
      setAuth(res.token, res.user)
      return res.user
    } finally {
      loading.value = false
    }
  }

  /** 应用启动时恢复登录态 */
  async function bootstrap() {
    await fetchMe()
  }

  return {
    token,
    user,
    loading,
    isAuthenticated,
    isAdmin,
    setAuth,
    setUser,
    logout,
    fetchMe,
    login,
    register,
    bootstrap,
  }
})
