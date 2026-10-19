import axios, {
  AxiosHeaders,
  type AxiosInstance,
  type AxiosRequestConfig,
  type InternalAxiosRequestConfig,
} from 'axios'

/** Fixed to /forum/api in production; provided by .env in development, defaulting to /forum/api */
export const BASE_URL = import.meta.env.VITE_API_BASE_URL || '/forum/api'

const http: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: 30000,
  headers: { 'Content-Type': 'application/json' },
})

/** Clear login state and redirect to the login page on 401 */
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
  /** When true, do not auto-redirect to login on 401 (used for identity probing at startup) */
  skipAuthRedirect?: boolean
}

interface ApiEnvelope<T> {
  code: number
  message: string
  data: T
}

/** Business error: keeps the HTTP status and structured backend data (e.g. missing gate conditions) */
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
        throw new ApiError(env.message || 'Request failed', env.code, env.data)
      }
      return env.data
    }
    return body as T
  } catch (err) {
    if (err instanceof ApiError) throw err
    // Normalize error messages so pages can display them directly
    const anyErr = err as {
      response?: { data?: { message?: string; data?: unknown }; status?: number }
      message?: string
    }
    const serverMsg = anyErr?.response?.data?.message
    if (serverMsg) {
      throw new ApiError(serverMsg, anyErr.response?.status, anyErr.response?.data?.data)
    }
    if (err instanceof Error) throw err
    throw new Error('Request failed, please try again later')
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
