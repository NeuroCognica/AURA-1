export {}

declare global {
  interface LauncherAPI {
    status(): Promise<{ services: Record<string, string> }>
    start(service: string): Promise<{ ok: boolean; service?: string; status?: string; reason?: string }>
    stop(service: string): Promise<{ ok: boolean; service?: string; status?: string; reason?: string }>
    startBackend?: () => Promise<any>
    stopBackend?: () => Promise<any>
    startFrontend?: () => Promise<any>
    stopFrontend?: () => Promise<any>
    killAll?: () => Promise<any>
    on?: (channel: string, cb: (...args: any[]) => void) => (() => void)
  }

  interface Window {
    launcher?: LauncherAPI
  }
}
