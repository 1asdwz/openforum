import type { GateCondition, GateRule } from '@/types'

export const GATE_FIELD_LABELS: Record<string, string> = {
  REAL_NAME: 'Real name',
  GENDER: 'Gender',
  AGE: 'Age',
  COUNTRY: 'Country/region',
  CITY: 'City',
  OCCUPATION: 'Occupation',
  EDUCATION: 'Education',
  PROPERTY_AREA: 'Property area',
  PROPERTY_COUNT: 'Property count',
  INTERESTS: 'Interests',
}

export const GATE_AUTH_LEVEL_LABELS: Record<string, string> = {
  LOW: 'Low',
  MEDIUM: 'Medium',
  HIGH: 'High',
  EXTREME: 'Very high',
}

const EDUCATION_LABELS: Record<string, string> = {
  HIGH_SCHOOL: 'High school or below',
  COLLEGE: 'Associate degree',
  BACHELOR: "Bachelor's degree",
  MASTER: "Master's degree",
  DOCTOR: 'Doctorate',
}

function valueLabel(field: string, value: string): string {
  if (field === 'EDUCATION') return EDUCATION_LABELS[value] || value
  return value
}

/** Readable text for a single condition */
export function gateConditionText(condition: GateCondition): string {
  const field = GATE_FIELD_LABELS[condition.field] || condition.field
  const labels = condition.labels?.length ? condition.labels : condition.values
  const list = (labels || []).map((v) => valueLabel(condition.field, v)).join(' / ')
  let text = ''
  switch (condition.field) {
    case 'AGE':
    case 'PROPERTY_COUNT': {
      const unit = condition.field === 'AGE' ? 'years' : 'properties'
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
      if (condition.op === 'not_in') text = `Not in ${text}`
      break
  }
  if (condition.authLevel && condition.field !== 'INTERESTS') {
    const level = GATE_AUTH_LEVEL_LABELS[condition.authLevel] || condition.authLevel
    text = `${text} (verification ≥ ${level})`
  }
  return `${field}: ${text}`
}

/** Readable summary of gate rules; returns an empty array for empty rules */
export function gateSummary(rule?: GateRule | null): string[] {
  if (!rule?.conditions?.length) return []
  return rule.conditions.map(gateConditionText)
}
