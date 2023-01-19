// Preload script for Electron
export interface IElectronAPI {
  get: (key: string, d: any) => any
  set: (key: string, value: any) => void
  onDidChange: (
    key: string,
    callback: (newValue: any, oldValue: any) => void
  ) => void
  invoke: (channel: string, ...args: any[]) => Promise<any>
  send: (channel: string, ...args: any[]) => void
  register: (name: string, callback: (...args: any[]) => void) => void
}

declare global {
  interface Window {
    electron: IElectronAPI
    alpine: any
  }
}
