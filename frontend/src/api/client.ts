import axios, {
  AxiosHeaders,
  type AxiosInstance,
  type AxiosRequestConfig,
  type InternalAxiosRequestConfig,
} from 'axios'

/** 生产环境固定为 /forum/api；开发环境由 .env 提供，默认 /forum/api */
export const BASE_URL = import.meta.env.VITE_API_BASE_URL || '/forum/api'

const http: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' },
})

/** 401 时清理登录态并跳转登录页 */
function handleUnauthorized() {
  localStorage.removeItem('token')
  localStorage.removeItem('user')
  const base = import.meta.env.BASE_URL || '/forum/'
  const loginPath = `${base.replace(/\/$/, '')}/login`
  if (!window.location.pathname.startsWith(loginPath)) {
    window.location.href = loginPath
  }
}

http.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = localStorage.getItem('token')
  if (token) {
    if (!config.headers) config.headers = new AxiosHeaders()
    ;(config.headers as AxiosHeaders).set('Authorization', `Bearer ${token}`)
  }
  return config
})

http.interceptors.response.use(
  (response) => response,
  (error) => {
    const status = error?.response?.status
    const skip = (error?.config as RequestConfigExtra)?.skipAuthRedirect
    if (status === 401 && !skip) {
      handleUnauthorized()
    }
    return Promise.reject(error)
  }
)

interface RequestConfigExtra extends AxiosRequestConfig {
  /** 为 true 时不因 401 自动跳转登录（用于启动时的身份探测） */
  skipAuthRedirect?: boolean
}

interface ApiEnvelope<T> {
  code: number
  message: string
  data: T
}

/** 业务错误：保留 HTTP 状态与后端结构化 data（如门槛缺失条件） */
export class ApiError extends Error {
  status?: number
  data?: unknown

  constructor(message: string, status?: number, data?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.data = data
  }
}

function isEnvelope(body: unknown): body is ApiEnvelope<unknown> {
  return !!body && typeof body === 'object' && 'code' in (body as object)
}

async function request<T>(config: RequestConfigExtra): Promise<T> {
  try {
    const res = await http.request<ApiEnvelope<T> | T>(config)
    const body = res.data as unknown
    if (isEnvelope(body)) {
      const env = body as ApiEnvelope<T>
      if (env.code !== 200 && env.code !== 0) {
        if (env.code === 401) handleUnauthorized()
        throw new ApiError(env.message || '请求失败', env.code, env.data)
      }
      return env.data
    }
    return body as T
  } catch (err) {
    if (err instanceof ApiError) throw err
    // 归一化错误信息，方便页面直接展示
    const anyErr = err as {
      response?: { data?: { message?: string; data?: unknown }; status?: number }
      message?: string
    }
    const serverMsg = anyErr?.response?.data?.message
    if (serverMsg) {
      throw new ApiError(serverMsg, anyErr.response?.status, anyErr.response?.data?.data)
    }
    if (err instanceof Error) throw err
    throw new Error('请求失败，请稍后重试')
  }
}

export const api = {
  get: <T>(url: string, config?: RequestConfigExtra) =>
    request<T>({ ...config, url, method: 'GET' }),
  post: <T>(url: string, data?: unknown, config?: RequestConfigExtra) =>
    request<T>({ ...config, url, method: 'POST', data }),
  put: <T>(url: string, data?: unknown, config?: RequestConfigExtra) =>
    request<T>({ ...config, url, method: 'PUT', data }),
  delete: <T>(url: string, config?: RequestConfigExtra) =>
    request<T>({ ...config, url, method: 'DELETE' }),
}

export default http
