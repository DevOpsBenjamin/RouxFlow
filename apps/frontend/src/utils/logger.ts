const isDev = import.meta.env.DEV

export const logger = {
  debug: (message: string, ...args: any[]) => {
    if (isDev) console.log(`[DEBUG] ${message}`, ...args)
  },
  info: (message: string, ...args: any[]) => {
    if (isDev) console.log(`[INFO] ${message}`, ...args)
  },
  warn: (message: string, ...args: any[]) => {
    console.warn(`[WARN] ${message}`, ...args)
  },
  error: (message: string, ...args: any[]) => {
    console.error(`[ERROR] ${message}`, ...args)
  }
}
