import { api, BASE_URL } from './client'
import type {
  AuthChallenge,
  AuthRecord,
  AuthResult,
  AuthTypeMeta,
  AuthenticationSubmitPayload,
  Comment,
  DeveloperApp,
  DeveloperAppList,
  ForumHomepageData,
  ForumHomepageMode,
  ForumMember,
  ForumModerator,
  ForumPermission,
  GateRule,
  HomepageData,
  HomepageMode,
  InfoFieldGroup,
  MediaKind,
  MediaUploadResult,
  MyForumHomepage,
  MyHomepage,
  OAuthAuthorizeInfo,
  Post,
  PublicForumHomepage,
  PublicHomepage,
  SubForum,
  User,
  UserInfoResponse,
} from '@/types'

/** Frontend paging starts at 1 while backend Spring Pageable starts at 0; convert here. */
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
  browseRule?: GateRule | null
  operateRule?: GateRule | null
}

export const authApi = {
  login: (payload: LoginPayload) =>
    api.post<AuthResult>('/auth/login', payload),
  register: (payload: RegisterPayload) =>
    api.post<AuthResult>('/auth/register', payload),
  /** Sign in / sign up with a Google ID token */
  google: (credential: string) =>
    api.post<AuthResult>('/auth/google', { credential }),
  me: () => api.get<User>('/auth/me', { skipAuthRedirect: true }),
}

export const userApi = {
  get: (id: number | string) => api.get<User>(`/users/${id}`),
  posts: (id: number | string, params: { sort?: 'hot' | 'new' | 'top'; page?: number; size?: number } = {}) =>
    api.get<unknown>(`/users/${id}/posts`, { params: shiftPage(params) }),
  comments: (id: number | string, params: { page?: number; size?: number } = {}) =>
    api.get<unknown>(`/users/${id}/comments`, { params: shiftPage(params) }),
  updateMe: (payload: { avatar?: string; bio?: string }) =>
    api.put<User>('/users/me', payload),
}

export const homepageApi = {
  /** Publicly read a user's homepage */
  get: (id: number | string) => api.get<PublicHomepage>(`/users/${id}/homepage`),
  /** Public profile, stats, and read-only lists required by data components */
  data: (id: number | string) =>
    api.get<HomepageData>(`/users/${id}/homepage-data`, { skipAuthRedirect: true }),
  /** Read your own homepage configuration */
  mine: () => api.get<MyHomepage>('/users/me/homepage'),
  /** Save mode and custom HTML */
  save: (payload: { mode: HomepageMode; html?: string }) =>
    api.put<MyHomepage>('/users/me/homepage', payload),
  /** Reset to the normal homepage and clear report records */
  reset: () => api.post<MyHomepage>('/users/me/homepage/reset'),
  /** Report another user's custom homepage */
  report: (id: number | string, reason: string) =>
    api.post<{ ok: boolean }>(`/users/${id}/homepage/report`, { reason }),
}

export const infoApi = {
  /** Info field catalog (groups + field metadata) */
  fields: () => api.get<InfoFieldGroup[]>('/info/fields'),
  /** My profile (including completeness) */
  me: () => api.get<UserInfoResponse>('/info/me'),
  /** Fill in / update a single info item */
  update: (fieldKey: string, value: Record<string, unknown>) =>
    api.put<UserInfoResponse['groups'][number]['items'][number]>(
      `/info/me/${fieldKey}`,
      { value }
    ),
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
  /** Publicly read forum homepage mode and HTML (after passing the browse gate) */
  homepage: (id: number | string) =>
    api.get<PublicForumHomepage>(`/forums/${id}/homepage`),
  /** Manager reads the full homepage configuration */
  myHomepage: (id: number | string) =>
    api.get<MyForumHomepage>(`/forums/${id}/homepage/mine`),
  /** Save mode and custom HTML */
  saveHomepage: (id: number | string, payload: { mode: ForumHomepageMode; html?: string }) =>
    api.put<MyForumHomepage>(`/forums/${id}/homepage`, payload),
  /** Reset to the normal homepage and clear report records */
  resetHomepage: (id: number | string) =>
    api.post<MyForumHomepage>(`/forums/${id}/homepage/reset`),
  /** Report another forum's custom homepage */
  reportHomepage: (id: number | string, reason: string) =>
    api.post<{ ok: boolean }>(`/forums/${id}/homepage/report`, { reason }),
  /** Public info, stats, and read-only lists required by data components */
  homepageData: (id: number | string) =>
    api.get<ForumHomepageData>(`/forums/${id}/homepage-data`, { skipAuthRedirect: true }),
  join: (id: number | string) => api.post<unknown>(`/forums/${id}/join`),
  leave: (id: number | string) => api.post<unknown>(`/forums/${id}/leave`),
  members: (id: number | string) =>
    api.get<ForumMember[]>(`/forums/${id}/members`),
  /** Moderators and permission list */
  moderators: (id: number | string) =>
    api.get<ForumModerator[]>(`/forums/${id}/moderators`),
  /** Invite / promote to moderator */
  addModerator: (
    id: number | string,
    payload: { username: string; permissions: ForumPermission[] }
  ) => api.post<ForumModerator>(`/forums/${id}/moderators`, payload),
  /** Update moderator permissions */
  updateModerator: (
    id: number | string,
    userId: number,
    permissions: ForumPermission[]
  ) => api.put<ForumModerator>(`/forums/${id}/moderators/${userId}`, { permissions }),
  /** Remove a moderator (demote back to a regular member) */
  removeModerator: (id: number | string, userId: number) =>
    api.delete<unknown>(`/forums/${id}/moderators/${userId}`),
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
  /** Upload post media (requires login), returns the media id */
  upload: (file: File) => {
    const form = new FormData()
    form.append('file', file)
    return api.post<MediaUploadResult>('/media', form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },
  /** Public media URL */
  url: (id: number | string) => `${BASE_URL}/media/${id}`,
}

export const authenticationApi = {
  types: () => api.get<AuthTypeMeta[]>('/authentications/types'),
  /** Get a one-time liveness challenge (random action sequence) */
  challenge: () => api.get<AuthChallenge>('/authentications/challenge'),
  /** Upload an ID/selfie/video, returns the media id */
  uploadMedia: (file: File, kind: MediaKind) => {
    const form = new FormData()
    form.append('file', file)
    form.append('kind', kind)
    return api.post<AuthMediaUploadResult>('/authentications/media', form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  },
  /** Media URL (requires auth; pages fetch it with mediaBlob and a token) */
  mediaUrl: (id: number | string) => `${BASE_URL}/authentications/media/${id}`,
  /** Fetch media content with auth (for thumbnails/previews) */
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
  users: () => api.get<unknown>('/admin/users'),
  forumHomepages: (params: { status?: string; page?: number; size?: number } = {}) =>
    api.get<unknown>('/admin/forum-homepages', { params: shiftPage(params) }),
  blockForumHomepage: (forumId: number | string, reason: string) =>
    api.post<{ ok: boolean }>(`/admin/forum-homepages/${forumId}/block`, { reason }),
  unblockForumHomepage: (forumId: number | string) =>
    api.post<{ ok: boolean }>(`/admin/forum-homepages/${forumId}/unblock`),
  userHomepages: (params: { status?: string; page?: number; size?: number } = {}) =>
    api.get<unknown>('/admin/user-homepages', { params: shiftPage(params) }),
  blockUserHomepage: (userId: number | string, reason: string) =>
    api.post<{ ok: boolean }>(`/admin/user-homepages/${userId}/block`, { reason }),
  unblockUserHomepage: (userId: number | string) =>
    api.post<{ ok: boolean }>(`/admin/user-homepages/${userId}/unblock`),
}

export interface DeveloperAppPayload {
  name: string
  description?: string
  homepageUrl?: string
  redirectUris: string[]
  clientType?: 'CONFIDENTIAL' | 'PUBLIC'
}

export const developerApi = {
  list: () => api.get<DeveloperAppList>('/developers/apps'),
  create: (payload: DeveloperAppPayload) =>
    api.post<DeveloperApp>('/developers/apps', payload),
  get: (id: number | string) => api.get<DeveloperApp>(`/developers/apps/${id}`),
  update: (
    id: number | string,
    payload: Partial<DeveloperAppPayload> & { status?: 'ACTIVE' | 'DISABLED' }
  ) => api.put<DeveloperApp>(`/developers/apps/${id}`, payload),
  remove: (id: number | string) => api.delete<unknown>(`/developers/apps/${id}`),
  resetSecret: (id: number | string) =>
    api.post<DeveloperApp>(`/developers/apps/${id}/secret`),
  /** Public OpenAPI document URL */
  specUrl: () => `${BASE_URL}/developers/docs`,
}

export const oauthApi = {
  authorizeInfo: (params: {
    client_id: string
    redirect_uri?: string
    scope?: string
  }) => api.get<OAuthAuthorizeInfo>('/oauth/authorize/info', { params }),
  authorize: (payload: {
    clientId: string
    redirectUri: string
    state?: string
    codeChallenge?: string
    codeChallengeMethod?: string
    scope?: string
    approve: boolean
  }) => api.post<{ redirectTo: string }>('/oauth/authorize', payload),
}
