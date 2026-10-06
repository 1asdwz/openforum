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

export interface SubForum {
  id: number
  slug: string
  name: string
  description?: string | null
  icon?: string | null
  themeColor?: string | null
  rules?: string | null
  requiredAuthTypes?: string | string[] | null
  templateType: TemplateType
  templateConfig?: string | TemplateConfig | null
  customPageHtml?: string | null
  customPageStatus?: CustomPageStatus
  customPageReviewNote?: string | null
  memberCount: number
  postCount: number
  isMember?: boolean
  canPost?: boolean
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
