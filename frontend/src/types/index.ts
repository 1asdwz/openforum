export type Role = 'USER' | 'ADMIN'

export interface User {
  id: number
  username: string
  email: string
  avatar?: string | null
  bio?: string | null
  role: Role
  createdAt?: string
  approvedAuthTypes?: string[]
}

export interface AuthResult {
  token: string
  user: User
}

export type TemplateType = 'DEFAULT' | 'CUSTOM'
export type CustomPageStatus = 'DRAFT' | 'PENDING' | 'APPROVED' | 'REJECTED'

/** 门槛条件（与后端 GateCondition 对齐） */
export interface GateCondition {
  field: string
  op?: string
  values?: string[]
  labels?: string[]
  value?: string
  min?: number
  max?: number
  authLevel?: string
}

export interface GateRule {
  version: number
  conditions: GateCondition[]
}

export interface GateMissingItem {
  fieldKey: string
  label: string
  reason: 'NOT_FILLED' | 'VALUE_MISMATCH' | 'AUTH_PENDING' | 'AUTH_LEVEL_LOW' | string
  required: string
}

/** 403 门槛拦截附带的数据 */
export interface GateDeniedData {
  gate: 'BROWSE' | 'OPERATE' | string
  forumId?: number
  forumName?: string
  loginRequired?: boolean
  missing: GateMissingItem[]
  /** 前端补充的错误文案 */
  message?: string
}

export interface SubForum {
  id: number
  slug: string
  name: string
  description?: string | null
  icon?: string | null
  themeColor?: string | null
  rules?: string | null
  /** 旧版认证门槛（仅兼容展示） */
  requiredAuthTypes?: string | string[] | null
  legacyRequiredAuthTypes?: string | null
  browseRule?: GateRule | null
  operateRule?: GateRule | null
  templateType: TemplateType
  templateConfig?: string | TemplateConfig | null
  customPageHtml?: string | null
  customPageStatus?: CustomPageStatus
  customPageReviewNote?: string | null
  memberCount: number
  postCount: number
  isMember?: boolean
  canPost?: boolean
  canOperate?: boolean
  creatorId?: number
  createdAt?: string
}


export interface TemplateConfig {
  announcement?: string
  sections?: { title: string; content: string }[]
  links?: { label: string; url: string }[]
}

export interface ForumMember {
  id?: number
  userId: number
  username: string
  avatar?: string | null
  role?: string
  joinedAt?: string
}

export interface Post {
  id: number
  forumId: number
  forumSlug: string
  forumName: string
  authorId: number
  authorName: string
  title: string
  content: string
  upvotes: number
  downvotes: number
  score: number
  commentCount: number
  isPinned: boolean
  createdAt: string
  myVote?: number
  canVote?: boolean
}

export interface Comment {
  id: number
  postId: number
  parentId?: number | null
  authorId: number
  authorName: string
  content: string
  score: number
  upvotes: number
  downvotes: number
  createdAt: string
  replies: Comment[]
  myVote?: number
  canVote?: boolean
}

/** 用户主页「评论」列表：扁平评论 + 所属帖子标题 */
export interface UserComment {
  id: number
  postId: number
  parentId?: number | null
  authorId: number
  authorName: string
  content: string
  score: number
  upvotes: number
  downvotes: number
  createdAt: string
  postTitle?: string | null
}

export interface AuthFieldDef {
  name: string
  label: string
  type?: string
  required?: boolean
  options?: (string | { label: string; value: string })[]
}

export type AuthLevel = 'LOW' | 'MEDIUM' | 'HIGH' | 'EXTREME'

export interface AuthLevelMeta {
  level: AuthLevel
  label: string
  requirements: string[]
  tips: string[]
}

export interface AuthTypeMeta {
  type: string
  label: string
  description?: string
  fields: AuthFieldDef[]
  levels?: AuthLevelMeta[]
}

export type AuthStatus = 'PENDING' | 'APPROVED' | 'REJECTED'

export type MediaKind = 'DOCUMENT' | 'SELFIE' | 'VIDEO'

export interface AuthMediaInfo {
  id: number
  kind: MediaKind
  contentType?: string
  sizeBytes?: number
}

export interface MediaUploadResult {
  id: number
  contentType: string
  sizeBytes: number
}

export interface AuthChallenge {
  challengeId: string
  actions: string[]
  nonce: string
  expiresAt?: string
}

export interface AuthenticationSubmitPayload {
  type: string
  level: AuthLevel
  data: Record<string, unknown>
  mediaIds?: number[]
  videoMediaId?: number
  challengeId?: string
  liveness?: Record<string, unknown>
  attestation?: Record<string, unknown>
}

export interface AuthRecord {
  id: number
  type: string
  level?: AuthLevel
  status: AuthStatus
  authData?: string | null
  data?: Record<string, unknown> | null
  reviewMethod?: string
  confidence?: number
  reason?: string
  rejectionReason?: string
  media?: AuthMediaInfo[]
  liveness?: Record<string, unknown> | null
  attestation?: Record<string, unknown> | null
  createdAt?: string
  reviewedAt?: string
}

export interface CustomPageInfo {
  html?: string | null
  status?: CustomPageStatus
  reviewNote?: string | null
  customPageReviewNote?: string | null
}

export interface PageResult<T> {
  items: T[]
  total: number
}

/** 信息项目录 */
export interface InfoField {
  fieldKey: string
  label: string
  valueType: 'text' | 'enum' | 'date' | 'region' | 'ordinal' | 'region_multi' | 'int' | 'tags' | string
  options: { value: string; label: string; order?: number }[]
  authable: boolean
  gateOps: string[]
  authLevels: string[]
}

export interface InfoFieldGroup {
  groupKey: string
  groupLabel: string
  fields: InfoField[]
}

/** 我的资料项 */
export interface UserInfoItem {
  fieldKey: string
  label: string
  value: Record<string, unknown> | null
  valueText: string
  authLevel?: string | null
  authStatus?: string | null
  canUpgrade?: boolean
  privacy?: string
}

export interface UserInfoGroup {
  groupKey: string
  groupLabel: string
  items: UserInfoItem[]
}

export interface UserInfoResponse {
  completeness: { filled: number; total: number; percent: number }
  groups: UserInfoGroup[]
}

