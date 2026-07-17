import { startE2eServer } from './server-lifecycle.mjs'

export default async function globalSetup() {
  const server = await startE2eServer()
  return server.close
}
