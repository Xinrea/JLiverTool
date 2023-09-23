import zh from './zh'

export enum LanguageType {
  zh,
}

const L = new Map()
L[LanguageType.zh] = zh

export default L
