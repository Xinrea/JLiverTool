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
const appDataPath = app.getPath('appData')
const logFilePath = path.join(appDataPath, 'JLiverTool.log')

let dev: boolean = false
if (process.env.NODE_ENV) {
  dev = process.env.NODE_ENV.includes('development')
}

class JLogger {
  private static _logger: Logger
  private constructor() { }

  public static getInstance(): Logger {
    if (!JLogger._logger) {
      JLogger._logger = new Logger({
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
    }
    return JLogger._logger
  }
}

export default JLogger
