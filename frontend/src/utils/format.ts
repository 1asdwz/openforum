import type { PageResult, TemplateConfig } from '@/types'

/** 相对时间格式化 */
export function formatTime(input?: string | null): string {
  if (!input) return ''
  const d = new Date(input)
  if (Number.isNaN(d.getTime())) return input
  const diff = Date.now() - d.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} 小时前`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days} 天前`
  return d.toLocaleDateString('zh-CN')
}

/** 绝对时间格式化 */
export function formatDate(input?: string | null): string {
  if (!input) return ''
  const d = new Date(input)
  if (Number.isNaN(d.getTime())) return input
  return d.toLocaleString('zh-CN', { hour12: false })
}

export const AUTH_TYPE_LABELS: Record<string, string> = {
  REAL_NAME: '实名认证',
  ID_CARD: '身份证',
  GENDER: '性别',
  AGE: '年龄',
  NATIONALITY: '国籍',
  DRIVER_LICENSE: '驾驶证',
  MARRIAGE_CERT: '婚姻状况',
  PROPERTY_CERT: '房产证明',
  ASSETS: '资产证明',
  CUSTOM: '自定义',
}

/** 认证等级标签 */
export const AUTH_LEVEL_LABELS: Record<string, string> = {
  LOW: '低',
  MEDIUM: '中',
  HIGH: '高',
  EXTREME: '极高',
}

/** 等级中文名 */
export function authLevelLabel(level?: string | null): string {
  if (!level) return ''
  return AUTH_LEVEL_LABELS[level] || level
}

/** 将门槛项（TYPE / TYPE:LEVEL / AGE_n）转为中文标签 */
export function authTypeLabel(raw: string): string {
  const t = (raw || '').trim()
  if (!t) return ''
  if (t.startsWith('AGE_')) return `年龄 ≥ ${t.slice(4)}`
  const separator = t.indexOf(':')
  if (separator > 0) {
    const type = t.slice(0, separator)
    const level = t.slice(separator + 1)
    return `${AUTH_TYPE_LABELS[type] || type}（${authLevelLabel(level)}）`
  }
  return AUTH_TYPE_LABELS[t] || t
}

/** 解析逗号分隔或数组形式的 requiredAuthTypes */
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

/** 建议的认证类型顺序 */
export const AUTH_TYPE_ORDER = [
  'REAL_NAME',
  'ID_CARD',
  'GENDER',
  'AGE',
  'NATIONALITY',
  'DRIVER_LICENSE',
  'MARRIAGE_CERT',
  'PROPERTY_CERT',
  'ASSETS',
  'CUSTOM',
]

/** 容错解析 templateConfig（可能是 JSON 字符串或对象） */
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

/** 容错解析 JSON 字符串 */
export function safeJsonParse<T = unknown>(raw?: string | null): T | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

/** 归一化分页返回结构 */
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

/** 归一化为数组（兼容分页对象或裸数组） */
export function toArray<T>(payload: unknown): T[] {
  if (Array.isArray(payload)) return payload as T[]
  return normalizePage<T>(payload).items
}

/** 提取错误信息 */
export function errorMessage(err: unknown, fallback = '请求失败，请稍后重试'): string {
  if (err instanceof Error && err.message) return err.message
  if (typeof err === 'string') return err
  return fallback
}

/** 去除 Markdown 语法，提取用于摘要的纯文本 */
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

/** 取用户名首字母作为头像占位 */
export function avatarText(name?: string | null): string {
  if (!name) return '?'
  return name.trim().charAt(0).toUpperCase()
}

/** 判断图标字段是否为图片 URL（否则视为 emoji/文字图标） */
export function isImageUrl(value?: string | null): value is string {
  if (!value) return false
  return /^(https?:\/\/|data:image\/|\/)/i.test(value.trim())
}

/** 返回可用于 img / 头像 src 的图片 URL；非图片值（如 emoji）返回 undefined */
export function imageSrc(value?: string | null): string | undefined {
  return isImageUrl(value) ? value : undefined
}
