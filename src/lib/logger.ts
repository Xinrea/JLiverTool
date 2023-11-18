import { app, ipcMain } from 'electron'

import {
  consoleOutput,
  DEBUG,
  defaultFormatter,
  INFO,
  Logger,
} from '@jalik/logger'
// @ts-ignore
import fileOutput from '@jalik/logger/dist/outputs/fileOutput.js'
import path = require('path')
import { LogEventContext } from '@jalik/logger/dist/event'
import JEvent from './events'

// Initialize logger
const logPath = app.getPath('logs')
const logFilePath = path.join(logPath, 'JLiverTool.log')
const dev = process.env.DEBUG === 'true'

console.log('log file path:', logFilePath)

class JLogger {
  private constructor() {}

  public static getInstance(name: string): Logger {
    return new Logger({
      name,
      level: dev ? DEBUG : INFO,
      outputs: [
        eventOutput(),
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

  public static getLogPath(): string {
    return logPath
  }
}

function eventOutput() {
  return (event: any) => {
    ipcMain.emit(JEvent[JEvent.EVENT_LOG], defaultFormatter(event))
  }
}

export default JLogger
