export function isPortableProjectContentId(value, maxLength) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= maxLength
    && value.trim() === value
    && /^[A-Za-z0-9_.-]+$/.test(value)
}
