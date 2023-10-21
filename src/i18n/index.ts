import zh from './zh'

export enum LanguageType {
  zh,
}

export const Languages = new Map()
Languages[LanguageType.zh] = zh

export default Languages
