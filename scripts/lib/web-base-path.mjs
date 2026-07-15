export function normalizeWebBasePath(value) {
  if (!value || value === './') return '/'
  if (value.startsWith('http://') || value.startsWith('https://')) return value
  const withLeadingSlash = value.startsWith('/') ? value : `/${value}`
  return withLeadingSlash.endsWith('/') ? withLeadingSlash : `${withLeadingSlash}/`
}
