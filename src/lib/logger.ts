import { app } from 'electron'
import path = require('path')

import {
  consoleOutput,
  DEBUG,
  defaultFormatter,
  INFO,
  Logger,
  // @ts-ignore
} from '@jalik/logger'
// @ts-ignore
import fileOutput from '@jalik/logger/dist/outputs/fileOutput.js'

// Initialize logger
const logPath = app.getPath('logs')
const logFilePath = path.join(logPath, 'JLiverTool.log')

console.log('Log file path:', logFilePath)

let dev: boolean = false
if (process.env.NODE_ENV) {
  dev = process.env.NODE_ENV.includes('development')
}

class JLogger {
  private constructor() {}

  public static getInstance(name: string): Logger {
    const logger = new Logger({
      name,
      level: dev ? DEBUG : INFO,
      outputs: [
        consoleOutput(),
        fileOutput({
          // the logs destination file
          path: logFilePath,
          // the formatter to use
          formatter: defaultFormatter,
          // improve performances by flushing (writing) logs at interval
          // instead of writing logs every time
          flushInterval: 1000,
        }),
      ],
    })
    return logger
  }
}

export default JLogger
