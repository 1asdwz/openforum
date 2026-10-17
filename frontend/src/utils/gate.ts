import type { GateCondition, GateRule } from '@/types'

export const GATE_FIELD_LABELS: Record<string, string> = {
  REAL_NAME: '实名',
  GENDER: '性别',
  AGE: '年龄',
  COUNTRY: '国家/地区',
  CITY: '城市',
  OCCUPATION: '职业',
  EDUCATION: '学历',
  PROPERTY_AREA: '房产区域',
  PROPERTY_COUNT: '房产数量',
  INTERESTS: '兴趣',
}

export const GATE_AUTH_LEVEL_LABELS: Record<string, string> = {
  LOW: '低',
  MEDIUM: '中',
  HIGH: '高',
  EXTREME: '极高',
}

const EDUCATION_LABELS: Record<string, string> = {
  HIGH_SCHOOL: '高中及以下',
  COLLEGE: '大专',
  BACHELOR: '本科',
  MASTER: '硕士',
  DOCTOR: '博士',
}

function valueLabel(field: string, value: string): string {
  if (field === 'EDUCATION') return EDUCATION_LABELS[value] || value
  return value
}

/** 单条条件的可读文本 */
export function gateConditionText(condition: GateCondition): string {
  const field = GATE_FIELD_LABELS[condition.field] || condition.field
  const labels = condition.labels?.length ? condition.labels : condition.values
  const list = (labels || []).map((v) => valueLabel(condition.field, v)).join(' / ')
  let text = ''
  switch (condition.field) {
    case 'AGE':
    case 'PROPERTY_COUNT': {
      const unit = condition.field === 'AGE' ? '岁' : '套'
      if (condition.op === 'between') {
        text = `${condition.min ?? 0}-${condition.max ?? '∞'} ${unit}`
      } else if (condition.op === 'gte') {
        text = `≥ ${condition.min ?? 0} ${unit}`
      } else {
        text = `≤ ${condition.max ?? '∞'} ${unit}`
      }
      break
    }
    case 'EDUCATION':
      if (condition.op === 'gte') text = `≥ ${list}`
      else if (condition.op === 'lte') text = `≤ ${list}`
      else text = list
      break
    default:
      text = list || condition.value || ''
      if (condition.op === 'not_in') text = `不属于 ${text}`
      break
  }
  if (condition.authLevel && condition.field !== 'INTERESTS') {
    const level = GATE_AUTH_LEVEL_LABELS[condition.authLevel] || condition.authLevel
    text = `${text}（认证 ≥ ${level}）`
  }
  return `${field}：${text}`
}

/** 门槛规则的可读摘要；空规则返回空数组 */
export function gateSummary(rule?: GateRule | null): string[] {
  if (!rule?.conditions?.length) return []
  return rule.conditions.map(gateConditionText)
}
