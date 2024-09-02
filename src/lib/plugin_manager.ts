import { readFile } from 'fs/promises'
import { BrowserWindow } from 'electron'
import path = require('path')

const dev = process.env.DEBUG === 'true'

export class Plugin {
  id: string
  name: string
  author: string
  version: string
  url: string
  path: string
  index: string

  private _window: BrowserWindow = null

  constructor(path: string) {
    this.path = path
  }

  async load() {
    try {
      const meta_raw = await readFile(this.path + '/meta.json', 'utf8')
      const meta_info = JSON.parse(meta_raw)
      this.id = meta_info.id
      this.name = meta_info.name
      this.index = meta_info.index
      this.version = meta_info.version
      this.url = meta_info.url
      this.author = meta_info.author
      console.log(
        'load plugin [%s][%s]%s',
        this.author,
        this.name,
        this.version
      )
      // initialize plugin window
      this._window = new BrowserWindow({
        parent: null,
        minHeight: 200,
        minWidth: 200,
        frame: true,
        show: true,
        title: this.name,
        webPreferences: {
          preload: path.join(__dirname, 'plugin_preload.js'),
          webSecurity: false,
        },
      })
      this._window.loadFile(this.path + '/' + this.index)

      if (dev) {
        this._window.webContents.openDevTools()
      }
    } catch (error) {
      console.log('load plugin failed:', this.path, error)
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

  async add(plugin_path: string) {
    const new_plugin = new Plugin(plugin_path)
    await new_plugin.load()
    this.plugins.push(new_plugin)
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
}
