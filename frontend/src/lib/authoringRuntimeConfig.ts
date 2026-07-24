export interface AuthoringRuntimeEnvironment {
  MONOGATARI_AI_API_KEY?: string
  MONOGATARI_API_KEY?: string
}

export function resolveAuthoringApiKey(environment: AuthoringRuntimeEnvironment): string {
  return String(
    environment.MONOGATARI_AI_API_KEY
      || environment.MONOGATARI_API_KEY
      || '',
  ).trim()
}
