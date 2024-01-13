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
import JEvent from './events'

// Initialize logger
const logPath = app.getPath('logs')
const logFilePath = path.join(logPath, 'JLiverTool.log')

console.log('log file path:', logFilePath)

class JLogger {
  private constructor() {}
  private static loggerInstanceList: Logger[] = []
  private static level: string = INFO

  public static getInstance(name: string): Logger {
    const instance = new Logger({
      name,
      level: this.level,
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
    this.loggerInstanceList.push(instance)
    return instance
  }

  public static getLogPath(): string {
    return logPath
  }

  public static updateLogLevel(level: string): void {
    this.level = level
    this.loggerInstanceList.forEach((logger) => {
      logger.setLevel(level)
    })
  }
}

function eventOutput() {
  return (event: any) => {
    ipcMain.emit(JEvent[JEvent.EVENT_LOG], defaultFormatter(event))
  }
}

export default JLogger
