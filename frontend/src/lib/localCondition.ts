export type LocalConditionValue = number | string | boolean

export interface LocalConditionScope {
  context?: Record<string, LocalConditionValue>
  variables?: Record<string, LocalConditionValue>
  flags?: Record<string, boolean>
}

export interface LocalConditionResult {
  result: boolean
  supported: boolean
  error: string | null
}

export function evaluateLocalCondition(value: unknown, scope: LocalConditionScope = {}): LocalConditionResult {
  const condition = String(value ?? '').trim()
  if (!condition) return { result: false, supported: false, error: 'condition_empty' }
  try {
    return {
      result: evaluateExpression(condition, {
        context: scope.context ?? {},
        variables: scope.variables ?? {},
        flags: scope.flags ?? {},
      }),
      supported: true,
      error: null,
    }
  } catch (error) {
    return { result: false, supported: false, error: String(error) }
  }
}

function evaluateExpression(expression: string, scope: Required<LocalConditionScope>): boolean {
  const text = stripOuterParens(expression.trim())
  const orParts = splitExpression(text, '||')
  if (orParts.length > 1) return orParts.some((part) => evaluateExpression(part, scope))
  const andParts = splitExpression(text, '&&')
  if (andParts.length > 1) return andParts.every((part) => evaluateExpression(part, scope))
  if (text.startsWith('!')) return !evaluateExpression(text.slice(1), scope)

  const comparison = findComparison(text)
  if (comparison) {
    return compareValues(
      conditionValue(comparison.left, scope),
      conditionValue(comparison.right, scope),
      comparison.operator,
    )
  }
  return Boolean(conditionValue(text, scope))
}

function splitExpression(expression: string, operator: '&&' | '||'): string[] {
  const parts: string[] = []
  let depth = 0
  let quote = ''
  let start = 0
  for (let index = 0; index < expression.length; index += 1) {
    const char = expression[index]
    if (quote) {
      if (char === quote && expression[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth = Math.max(0, depth - 1)
    if (depth === 0 && expression.startsWith(operator, index)) {
      parts.push(expression.slice(start, index).trim())
      start = index + operator.length
      index += operator.length - 1
    }
  }
  if (parts.length === 0) return [expression.trim()]
  parts.push(expression.slice(start).trim())
  if (parts.some((part) => !part)) throw new Error('unsupported_condition_syntax')
  return parts
}

function findComparison(expression: string): { left: string; operator: string; right: string } | null {
  let depth = 0
  let quote = ''
  const operators = ['>=', '<=', '==', '!=', '>', '<']
  for (let index = 0; index < expression.length; index += 1) {
    const char = expression[index]
    if (quote) {
      if (char === quote && expression[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth = Math.max(0, depth - 1)
    if (depth !== 0) continue
    const operator = operators.find((candidate) => expression.startsWith(candidate, index))
    if (!operator) continue
    return {
      left: expression.slice(0, index).trim(),
      operator,
      right: expression.slice(index + operator.length).trim(),
    }
  }
  return null
}

function conditionValue(raw: string, scope: Required<LocalConditionScope>): LocalConditionValue {
  const text = stripOuterParens(raw.trim())
  if (/^true$/i.test(text)) return true
  if (/^false$/i.test(text)) return false
  if (/^-?\d+(?:\.\d+)?$/.test(text)) return Number(text)
  const quoted = text.match(/^(['"])(.*)\1$/)
  if (quoted) return quoted[2].replace(/\\(['"\\])/g, '$1')
  const variable = text.match(/^[A-Za-z_][A-Za-z0-9_]*$/)
  if (variable && Object.prototype.hasOwnProperty.call(scope.context, text)) return scope.context[text]
  const getVariable = text.match(/^getVariable\((['"])([A-Za-z0-9_.-]+)\1\)$/)
  if (getVariable && Object.prototype.hasOwnProperty.call(scope.variables, getVariable[2])) {
    return scope.variables[getVariable[2]]
  }
  const hasFlag = text.match(/^hasFlag\((['"])([A-Za-z0-9_.-]+)\1\)$/)
  if (hasFlag) return Boolean(scope.flags[hasFlag[2]])
  throw new Error(`unsupported_condition:${text}`)
}

function compareValues(left: LocalConditionValue, right: LocalConditionValue, operator: string): boolean {
  if (operator === '==' || operator === '!=') {
    const equal = left === right
    return operator === '==' ? equal : !equal
  }
  const leftNumber = numericValue(left)
  const rightNumber = numericValue(right)
  if (leftNumber === null || rightNumber === null) throw new Error('unsupported_non_numeric_comparison')
  if (operator === '>=') return leftNumber >= rightNumber
  if (operator === '<=') return leftNumber <= rightNumber
  if (operator === '>') return leftNumber > rightNumber
  if (operator === '<') return leftNumber < rightNumber
  return false
}

function stripOuterParens(value: string): string {
  let text = value.trim()
  while (text.startsWith('(') && text.endsWith(')') && outerParensWrapExpression(text)) {
    text = text.slice(1, -1).trim()
  }
  return text
}

function outerParensWrapExpression(value: string): boolean {
  let depth = 0
  let quote = ''
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index]
    if (quote) {
      if (char === quote && value[index - 1] !== '\\') quote = ''
      continue
    }
    if (char === '"' || char === "'") {
      quote = char
      continue
    }
    if (char === '(') depth += 1
    if (char === ')') depth -= 1
    if (depth === 0 && index < value.length - 1) return false
  }
  return depth === 0
}

function numericValue(value: unknown): number | null {
  if (typeof value === 'number') return Number.isFinite(value) ? value : null
  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : null
  }
  return null
}
