import type { Comment, PageResult, Post, TemplateConfig } from '@/types'

/** Relative time formatting */
export function formatTime(input?: string | null): string {
  if (!input) return ''
  const d = new Date(input)
  if (Number.isNaN(d.getTime())) return input
  const diff = Date.now() - d.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes} min ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hours ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days} days ago`
  return d.toLocaleDateString('en-US')
}

/** Absolute time formatting */
export function formatDate(input?: string | null): string {
  if (!input) return ''
  const d = new Date(input)
  if (Number.isNaN(d.getTime())) return input
  return d.toLocaleString('en-US', { hour12: false })
}

export const AUTH_TYPE_LABELS: Record<string, string> = {
  REAL_NAME: 'Real-name verification',
  ID_CARD: 'ID card',
  GENDER: 'Gender',
  AGE: 'Age',
  NATIONALITY: 'Nationality',
  CITY: 'City',
  OCCUPATION: 'Occupation',
  EDUCATION: 'Education',
  DRIVER_LICENSE: "Driver's license",
  MARRIAGE_CERT: 'Marital status',
  PROPERTY_CERT: 'Property certificate',
  ASSETS: 'Proof of assets',
  CUSTOM: 'Custom',
}

/** Auth level labels */
export const AUTH_LEVEL_LABELS: Record<string, string> = {
  LOW: 'Low',
  MEDIUM: 'Medium',
  HIGH: 'High',
  EXTREME: 'Very high',
}

/** Human-readable auth level label */
export function authLevelLabel(level?: string | null): string {
  if (!level) return ''
  return AUTH_LEVEL_LABELS[level] || level
}

/** Convert a gate item (TYPE / TYPE:LEVEL / AGE_n) into a readable label */
export function authTypeLabel(raw: string): string {
  const t = (raw || '').trim()
  if (!t) return ''
  if (t.startsWith('AGE_')) return `Age ≥ ${t.slice(4)}`
  const separator = t.indexOf(':')
  if (separator > 0) {
    const type = t.slice(0, separator)
    const level = t.slice(separator + 1)
    return `${AUTH_TYPE_LABELS[type] || type} (${authLevelLabel(level)})`
  }
  return AUTH_TYPE_LABELS[t] || t
}

/** Parse requiredAuthTypes given as a comma-separated string or an array */
export function parseRequiredAuthTypes(
  raw?: string | string[] | null
): string[] {
  if (!raw) return []
  if (Array.isArray(raw)) return raw.filter(Boolean)
  return raw
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean)
}

/** Recommended auth type order */
export const AUTH_TYPE_ORDER = [
  'REAL_NAME',
  'ID_CARD',
  'GENDER',
  'AGE',
  'NATIONALITY',
  'CITY',
  'OCCUPATION',
  'EDUCATION',
  'PROPERTY_CERT',
  'DRIVER_LICENSE',
  'MARRIAGE_CERT',
  'ASSETS',
  'CUSTOM',
]

/** Tolerantly parse templateConfig (may be a JSON string or an object) */
export function parseTemplateConfig(
  raw?: string | TemplateConfig | null
): TemplateConfig {
  if (!raw) return {}
  if (typeof raw === 'object') return raw
  try {
    const parsed = JSON.parse(raw)
    if (parsed && typeof parsed === 'object') return parsed as TemplateConfig
  } catch {
    /* ignore */
  }
  return {}
}

/** Tolerantly parse a JSON string */
export function safeJsonParse<T = unknown>(raw?: string | null): T | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

/** Normalize paginated response structures */
export function normalizePage<T>(payload: unknown): PageResult<T> {
  if (Array.isArray(payload)) {
    return { items: payload as T[], total: payload.length }
  }
  if (!payload || typeof payload !== 'object') {
    return { items: [], total: 0 }
  }
  const p = payload as Record<string, unknown>
  const raw =
    p.content ?? p.items ?? p.list ?? p.records ?? p.data ?? p.rows ?? []
  const items = Array.isArray(raw) ? (raw as T[]) : []
  const totalRaw =
    p.totalElements ?? p.total ?? p.count ?? p.totalCount ?? items.length
  const total = typeof totalRaw === 'number' ? totalRaw : items.length
  return { items, total }
}

/** Normalize to an array (handles paged objects or bare arrays) */
export function toArray<T>(payload: unknown): T[] {
  if (Array.isArray(payload)) return payload as T[]
  return normalizePage<T>(payload).items
}

/** Extract an error message */
export function errorMessage(err: unknown, fallback = 'Request failed, please try again later'): string {
  if (err instanceof Error && err.message) return err.message
  if (typeof err === 'string') return err
  return fallback
}

/** Strip Markdown syntax and return plain text for excerpts */
export function stripMarkdown(md?: string | null): string {
  if (!md) return ''
  let text = md
  text = text.replace(/```[\s\S]*?```/g, ' ')
  text = text.replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1')
  text = text.replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
  text = text.replace(/`([^`]*)`/g, '$1')
  text = text.replace(/^\s{0,3}>\s?/gm, '')
  text = text.replace(/^\s{0,3}#{1,6}\s+/gm, '')
  text = text.replace(/^\s{0,3}(?:[-*+]|\d+\.)\s+/gm, '')
  text = text.replace(/(\*\*|__)(.*?)\1/g, '$2')
  text = text.replace(/(\*|_)(.*?)\1/g, '$2')
  text = text.replace(/~~(.*?)~~/g, '$1')
  text = text.replace(/^\s{0,3}(?:[-*_]\s*){3,}$/gm, ' ')
  return text.replace(/\s+/g, ' ').trim()
}

/** Use the first character of the username as an avatar placeholder */
export function avatarText(name?: string | null): string {
  if (!name) return '?'
  return name.trim().charAt(0).toUpperCase()
}

/** Check whether the icon field is an image URL (otherwise treat it as emoji/text) */
export function isImageUrl(value?: string | null): value is string {
  if (!value) return false
  return /^(https?:\/\/|data:image\/|\/)/i.test(value.trim())
}

/** Return an image URL usable as img/avatar src; return undefined for non-image values (e.g. emoji) */
export function imageSrc(value?: string | null): string | undefined {
  return isImageUrl(value) ? value : undefined
}

/** Normalize backend PostSummaryDTO into frontend Post (filling in authorName / content, etc.) */
export function normalizePostSummary(raw: unknown): Post {
  const p = (raw ?? {}) as Record<string, unknown>
  const author = (p.author ?? {}) as Record<string, unknown>
  return {
    id: asNumber(p.id),
    forumId: asNumber(p.forumId),
    forumSlug: asString(p.forumSlug),
    forumName: asString(p.forumName),
    authorId: asNumber(p.authorId ?? author.id),
    authorName: asString(p.authorName ?? author.username),
    title: asString(p.title),
    content: asString(p.content ?? p.contentPreview),
    upvotes: asNumber(p.upvotes),
    downvotes: asNumber(p.downvotes),
    score: asNumber(p.score),
    commentCount: asNumber(p.commentCount),
    isPinned: p.isPinned === true,
    createdAt: asString(p.createdAt),
    myVote: typeof p.myVote === 'number' ? p.myVote : undefined,
    canVote: typeof p.canVote === 'boolean' ? p.canVote : undefined,
  }
}

/** Normalize backend CommentDTO (children) into frontend Comment (replies), recursing into child replies */
export function normalizeComment(raw: unknown): Comment {
  const c = (raw ?? {}) as Record<string, unknown>
  const children = Array.isArray(c.children)
    ? c.children
    : Array.isArray(c.replies)
      ? c.replies
      : []
  return {
    id: asNumber(c.id),
    postId: asNumber(c.postId),
    parentId: typeof c.parentId === 'number' ? c.parentId : null,
    authorId: asNumber(c.authorId),
    authorName: asString(c.authorName),
    content: asString(c.content),
    score: asNumber(c.score),
    upvotes: asNumber(c.upvotes),
    downvotes: asNumber(c.downvotes),
    createdAt: asString(c.createdAt),
    replies: children.map(normalizeComment),
    myVote: typeof c.myVote === 'number' ? c.myVote : undefined,
    canVote: typeof c.canVote === 'boolean' ? c.canVote : undefined,
  }
}

function asString(value: unknown): string {
  return typeof value === 'string' ? value : ''
}

function asNumber(value: unknown): number {
  return typeof value === 'number' ? value : 0
}
