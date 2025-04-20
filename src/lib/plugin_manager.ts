import { readFile } from 'fs/promises'
import { BrowserWindow } from 'electron'
import path = require('path')
import JEvent from './events'
import JLogger from './logger'

const log = JLogger.getInstance('plugin_manager')

const dev = process.env.DEBUG === 'true'

export class Plugin {
  id: string
  name: string
  author: string
  desc: string
  version: string
  url: string
  path: string
  index: string

  _window: BrowserWindow = null

  constructor(path: string) {
    this.path = path
  }

  async load(): Promise<boolean> {
    try {
      const meta_raw = await readFile(this.path + '/meta.json', 'utf8')
      const meta_info = JSON.parse(meta_raw)
      this.id = meta_info.id
      this.name = meta_info.name
      this.desc = meta_info.desc
      this.index = meta_info.index
      this.version = meta_info.version
      this.url = meta_info.url
      this.author = meta_info.author
      log.info('load plugin', {
        auther: this.author,
        name: this.name,
        version: this.version,
      })
      // initialize plugin window
      this._window = new BrowserWindow({
        parent: null,
        minHeight: 200,
        minWidth: 200,
        frame: true,
        show: true,
        title: this.name,
        alwaysOnTop: true,
        webPreferences: {
          preload: path.join(__dirname, 'plugin_preload.js'),
          webSecurity: false,
        },
      })
      this._window.loadFile(this.path + '/' + this.index)
      this._window.on('close', (e) => {
        this._window.hide()
        e.preventDefault()
      })

      if (dev) {
        this._window.webContents.openDevTools()
      }

      return true
    } catch (error) {
      log.error('load plugin failed', { path: this.path, error })
      return false
    }
  }

  showWindow() {
    if (this._window) {
      this._window.show()
    }
  }

  getName() {
    return this.name
  }
}

export default class PluginManager {
  plugins: Plugin[]

  constructor() {
    this.plugins = []
  }

  async add(plugin_path: string): Promise<boolean> {
    // check if plugin_path is valid
    if (!plugin_path) {
      log.error('invalid plugin path')
      return false
    }
    // check if plugin_path already loaded
    const plugin = this.plugins.find((v) => {
      return v.path == plugin_path
    })
    if (plugin) {
      log.error('plugin already loaded', { path: plugin_path })
      return false
    }
    const new_plugin = new Plugin(plugin_path)
    if (await new_plugin.load()) {
      this.plugins.push(new_plugin)
      return true
    }

    return false
  }

  async remove(plugin_path: string) {
    // check if plugin_path is valid
    if (!plugin_path) {
      log.error('invalid plugin path')
      return false
    }
    // check if plugin_path already loaded
    const plugin = this.plugins.find((v) => {
      return v.path == plugin_path
    })
    if (!plugin) {
      log.error('plugin not found', { path: plugin_path })
      return false
    }

    plugin._window.destroy()
    plugin._window = null
    // remove plugin
    this.plugins = this.plugins.filter((v) => {
      return v.path != plugin_path
    })
    return true
  }

  getPlugins() {
    return this.plugins
  }

  showPluginWindow(plugin_id: string) {
    // find plugin
    const target = this.plugins.find((v) => {
      return v.id == plugin_id
    })
    if (target) {
      target.showWindow()
    }
  }

  broadcast(channel: JEvent, arg: any) {
    this.plugins.forEach((plugin) => {
      if (plugin._window) {
        plugin._window.webContents.send(JEvent[channel], arg)
      }
    })
  }
}
