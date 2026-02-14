const isDev = import.meta.env.DEV

function ts(): string {
  return `+${(performance.now() / 1000).toFixed(3)}s`
}

export const logger = {
  debug: (message: string, ...args: any[]) => {
    if (isDev) console.log(`${ts()} [DEBUG] ${message}`, ...args)
  },
  info: (message: string, ...args: any[]) => {
    if (isDev) console.log(`${ts()} [INFO] ${message}`, ...args)
  },
  warn: (message: string, ...args: any[]) => {
    console.warn(`${ts()} [WARN] ${message}`, ...args)
  },
  error: (message: string, ...args: any[]) => {
    console.error(`${ts()} [ERROR] ${message}`, ...args)
  }
}
