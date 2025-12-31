export {}

declare global {
  interface LauncherAPI {
    status(): Promise<{ services: Record<string, string> }>
    start(service: string): Promise<{ ok: boolean; service?: string; status?: string; reason?: string }>
    stop(service: string): Promise<{ ok: boolean; service?: string; status?: string; reason?: string }>
  }

  interface Window {
    launcher?: LauncherAPI
  }
}
