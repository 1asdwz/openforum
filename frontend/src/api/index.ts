import { api, BASE_URL } from './client'
import type {
  AuthChallenge,
  AuthRecord,
  AuthResult,
  AuthTypeMeta,
  AuthenticationSubmitPayload,
  Comment,
  CustomPageInfo,
  ForumMember,
  MediaKind,
  MediaUploadResult,
  Post,
  SubForum,
  User,
} from '@/types'

/** 前端分页从 1 开始，后端 Spring Pageable 从 0 开始，统一在此转换 */
function shiftPage<T extends { page?: number }>(params: T): T {
  if (typeof params.page !== 'number') return params
  return { ...params, page: Math.max(0, params.page - 1) }
}

export interface LoginPayload {
  usernameOrEmail: string
  password: string
}
export interface RegisterPayload {
  username: string
  email: string
  password: string
}
export interface CreateForumPayload {
  slug: string
  name: string
  description?: string
  icon?: string
  themeColor?: string
  templateType?: string
  templateConfig?: string
  rules?: string
  requiredAuthTypes?: string[]
}

export const authApi = {
  login: (payload: LoginPayload) =>
    api.post<AuthResult>('/auth/login', payload),
  register: (payload: RegisterPayload) =>
    api.post<AuthResult>('/auth/register', payload),
  me: () => api.get<User>('/auth/me', { skipAuthRedirect: true }),
}

export const userApi = {
  get: (id: number | string) => api.get<User>(`/users/${id}`),
  updateMe: (payload: { avatar?: string; bio?: string }) =>
    api.put<User>('/users/me', payload),
}

export const forumApi = {
  list: (params: { page?: number; size?: number; sort?: string; q?: string } = {}) =>
    api.get<unknown>('/forums', { params: shiftPage(params) }),
  popular: (limit = 10) =>
    api.get<SubForum[]>('/forums/popular', { params: { limit } }),
  recommended: (limit = 10) =>
    api.get<SubForum[]>('/forums/recommended', { params: { limit } }),
  create: (payload: CreateForumPayload) =>
    api.post<SubForum>('/forums', payload),
  get: (slug: string) => api.get<SubForum>(`/forums/${slug}`),
  update: (id: number | string, payload: Partial<CreateForumPayload>) =>
    api.put<SubForum>(`/forums/${id}`, payload),
  getCustomPage: (id: number | string) =>
    api.get<CustomPageInfo>(`/forums/${id}/custom-page`),
  saveCustomPage: (id: number | string, html: string) =>
    api.put<CustomPageInfo>(`/forums/${id}/custom-page`, { html }),
  submitCustomPage: (id: number | string) =>
    api.post<CustomPageInfo>(`/forums/${id}/custom-page/submit`),
  join: (id: number | string) => api.post<unknown>(`/forums/${id}/join`),
  leave: (id: number | string) => api.post<unknown>(`/forums/${id}/leave`),
  members: (id: number | string) =>
    api.get<ForumMember[]>(`/forums/${id}/members`),
}

export const postApi = {
  list: (
    params: {
      sort?: 'hot' | 'new' | 'top'
      page?: number
      size?: number
      forumId?: number | string
      q?: string
    } = {}
  ) => api.get<unknown>('/posts', { params: shiftPage(params) }),
  get: (id: number | string) => api.get<Post>(`/posts/${id}`),
  create: (payload: { forumId: number; title: string; content: string }) =>
    api.post<Post>('/posts', payload),
  vote: (id: number | string, value: number) =>
    api.post<Post>(`/posts/${id}/vote`, { value }),
  pin: (id: number | string) => api.post<unknown>(`/posts/${id}/pin`),
  remove: (id: number | string) => api.delete<unknown>(`/posts/${id}`),
}

export const commentApi = {
  listByPost: (postId: number | string) =>
    api.get<Comment[]>(`/comments/post/${postId}`),
  create: (payload: { postId: number; content: string; parentId?: number | null }) =>
    api.post<Comment>('/comments', payload),
  remove: (id: number | string) => api.delete<unknown>(`/comments/${id}`),
  vote: (id: number | string, value: number) =>
    api.post<Comment>(`/comments/${id}/vote`, { value }),
}

export interface AuthMediaUploadResult {
  mediaId: number
  kind: MediaKind
  contentType: string
  sizeBytes: number
}

export const mediaApi = {
  /** 上传帖子媒体（需登录），返回媒体 id */
  upload: (file: File) => {
    const form = new FormData()
    form.append('file', file)
    return api.post<MediaUploadResult>('/media', form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },
  /** 媒体公开访问地址 */
  url: (id: number | string) => `${BASE_URL}/media/${id}`,
}

export const authenticationApi = {
  types: () => api.get<AuthTypeMeta[]>('/authentications/types'),
  /** 获取一次性活体检测挑战（随机动作序列） */
  challenge: () => api.get<AuthChallenge>('/authentications/challenge'),
  /** 上传证件/自拍/视频，返回媒体 id */
  uploadMedia: (file: File, kind: MediaKind) => {
    const form = new FormData()
    form.append('file', file)
    form.append('kind', kind)
    return api.post<AuthMediaUploadResult>('/authentications/media', form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },
  /** 媒体访问地址（需鉴权，页面用 mediaBlob 带 token 获取） */
  mediaUrl: (id: number | string) => `${BASE_URL}/authentications/media/${id}`,
  /** 带鉴权获取媒体内容（用于缩略图/预览） */
  mediaBlob: (id: number | string) =>
    api.get<Blob>(`/authentications/media/${id}`, { responseType: 'blob' }),
  submit: (payload: AuthenticationSubmitPayload) =>
    api.post<AuthRecord>('/authentications', payload),
  mine: () => api.get<AuthRecord[]>('/authentications/me'),
  retry: (id: number | string, payload?: Partial<AuthenticationSubmitPayload>) =>
    api.post<AuthRecord>(`/authentications/${id}/retry`, payload),
}

export const adminApi = {
  authentications: (params: { status?: string; level?: string; page?: number; size?: number } = {}) =>
    api.get<unknown>('/admin/authentications', { params: shiftPage(params) }),
  approveAuthentication: (id: number | string) =>
    api.post<unknown>(`/admin/authentications/${id}/approve`),
  rejectAuthentication: (id: number | string, reason?: string) =>
    api.post<unknown>(`/admin/authentications/${id}/reject`, { reason }),
  customPages: (params: { status?: string; page?: number; size?: number } = {}) =>
    api.get<unknown>('/admin/custom-pages', { params: shiftPage(params) }),
  approveCustomPage: (id: number | string) =>
    api.post<unknown>(`/admin/custom-pages/${id}/approve`),
  rejectCustomPage: (id: number | string, reason?: string) =>
    api.post<unknown>(`/admin/custom-pages/${id}/reject`, { reason }),
  users: () => api.get<unknown>('/admin/users'),
}
