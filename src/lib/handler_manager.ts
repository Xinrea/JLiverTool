import { ipcMain, shell } from 'electron'
import { InitHandlerOpt } from './types'

export class HandlerManager {
  private static _instance: HandlerManager

  private constructor() {
    ipcMain.handle('showItemInFolder', (_, file_path) => {
      shell.showItemInFolder(file_path)
    })
    ipcMain.handle('openURL', (_, url) => {
      require('openurl').open(url)
    })
  }

  public static get Instance() {
    return this._instance || (this._instance = new this())
  }

  public initPathRelated(opt: InitHandlerOpt) {
    ipcMain.handle('getPath', (_, name: string) => {
      switch (name) {
        case 'appData': {
          return opt.appDataPath
        }
        case 'log': {
          return opt.logFilePath
        }
        default: {
          return 'UnknownName'
        }
      }
    })
  }
}
