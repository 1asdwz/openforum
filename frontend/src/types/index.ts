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

/** Gate conditions (aligned with backend GateCondition) */
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

/** Data attached to a 403 gate rejection */
export interface GateDeniedData {
  gate: 'BROWSE' | 'OPERATE' | string
  forumId?: number
  forumName?: string
  loginRequired?: boolean
  missing: GateMissingItem[]
  /** Error message added by the frontend */
  message?: string
}

/** Moderation permissions, matching Reddit's moderator permissions dialog */
export type ForumPermission =
  | 'EVERYTHING'
  | 'USERS'
  | 'CONFIG'
  | 'FLAIR'
  | 'MAIL'
  | 'POSTS_COMMENTS'
  | 'WIKI'

export interface SubForum {
  id: number
  slug: string
  name: string
  description?: string | null
  icon?: string | null
  themeColor?: string | null
  rules?: string | null
  /** Legacy auth gate (display compatibility only) */
  requiredAuthTypes?: string | string[] | null
  legacyRequiredAuthTypes?: string | null
  browseRule?: GateRule | null
  operateRule?: GateRule | null
  templateType: TemplateType
  templateConfig?: string | TemplateConfig | null
  memberCount: number
  postCount: number
  isMember?: boolean
  canPost?: boolean
  canOperate?: boolean
  canManage?: boolean
  myPermissions?: ForumPermission[]
  isCreator?: boolean
  creatorId?: number
  creatorName?: string | null
  createdAt?: string
}

/** Forum moderators and permission details */
export interface ForumModerator {
  userId: number
  username: string
  avatar?: string | null
  role: 'OWNER' | 'MODERATOR' | string
  isCreator: boolean
  permissions: ForumPermission[]
  joinedAt?: string
  promotedAt?: string | null
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

/** User profile "comments" list: flat comments plus their post titles */
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

export interface PageResult<T> {
  items: T[]
  total: number
}

export type HomepageMode = 'NORMAL' | 'CUSTOM'
export type HomepageStatus = 'ACTIVE' | 'BLOCKED'

/** Public homepage view: html is returned only when mode is CUSTOM; when blocked, others receive NORMAL only */
export interface PublicHomepage {
  mode: HomepageMode
  html?: string
  blocked?: boolean
  blockReason?: string | null
  updatedAt?: string | null
}

/** Own homepage configuration */
export interface MyHomepage {
  mode: HomepageMode
  html: string | null
  status: HomepageStatus
  blockReason: string | null
  sizeBytes: number
  updatedAt: string | null
}

/** Post entry used by custom homepage data components */
export interface HomepageDataPost {
  id: number
  title: string
  forumName: string
  forumSlug: string
  score: number
  commentCount: number
  createdAt: string
}

/** Public data required by custom homepage data components */
export interface HomepageData {
  profile: {
    id: number
    username: string
    avatar: string | null
    bio: string | null
    role: Role
    createdAt: string
  }
  stats: { posts: number; comments: number; upvotes: number }
  posts: HomepageDataPost[]
  liked: HomepageDataPost[]
  commented: HomepageDataPost[]
}

/** User homepage entry in the admin console */
export interface AdminHomepageItem {
  userId: number
  username: string
  mode: HomepageMode
  status: HomepageStatus
  sizeBytes: number
  blockReason?: string | null
  reportCount: number
  updatedAt?: string | null
}

export type ForumHomepageMode = 'NORMAL' | 'CUSTOM'

/** Public forum homepage view: visitors receive NORMAL only when blocked */
export interface PublicForumHomepage {
  mode: ForumHomepageMode
  html?: string
  blocked?: boolean
  blockReason?: string | null
  updatedAt?: string | null
}

/** Full forum homepage configuration read by managers */
export interface MyForumHomepage {
  mode: ForumHomepageMode
  html: string | null
  status: HomepageStatus
  blockReason: string | null
  sizeBytes: number
  updatedAt: string | null
}

/** Post entry used by custom forum homepage components */
export interface ForumHomepagePost {
  id: number
  title: string
  score: number
  commentCount: number
  authorName: string | null
  createdAt: string
}

/** Member entry used by custom forum homepage components */
export interface ForumMemberSummary {
  id: number
  username: string
  avatar: string | null
  role: string
  joinedAt: string
}

/** Public data required by forum homepage data components */
export interface ForumHomepageData {
  forum: {
    id: number
    slug: string
    name: string
    description: string | null
    icon: string | null
    themeColor: string | null
    createdAt: string
  }
  stats: { members: number; posts: number; comments: number }
  posts: ForumHomepagePost[]
  hotPosts: ForumHomepagePost[]
  members: ForumMemberSummary[]
}

/** Forum homepage entry in the admin console */
export interface AdminForumHomepageItem {
  forumId: number
  forumName: string
  forumSlug: string
  creatorName: string
  status: HomepageStatus
  sizeBytes: number
  blockReason?: string | null
  reportCount: number
  updatedAt?: string | null
}

/** Info field catalog */
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

/** My profile item */
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

export type OAuthClientType = 'CONFIDENTIAL' | 'PUBLIC'
export type OAuthAppStatus = 'ACTIVE' | 'DISABLED'

/** A third-party application registered in the developer center */
export interface DeveloperApp {
  id: number
  clientId: string
  name: string
  description?: string | null
  homepageUrl?: string | null
  redirectUris: string[]
  clientType: OAuthClientType
  scopes: string
  status: OAuthAppStatus
  hasSecret: boolean
  /** Only present in create / secret-reset responses */
  clientSecret?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface DeveloperAppList {
  content: DeveloperApp[]
  maxApps: number
  maxRedirectUris: number
}

/** Public description of an app shown on the consent screen */
export interface OAuthAuthorizeInfo {
  clientId: string
  name: string
  description?: string | null
  homepageUrl?: string | null
  ownerUsername: string
  clientType: OAuthClientType
  scope: string
  redirectUri: string
}

