// 应用配置接口
export interface AppConfig {
  rootDir: string
}

// 迁移结果接口
export interface MigrationResult {
  success: boolean
  message: string
  databaseCopied: boolean
  imagesCopied: number
  pathsUpdated: number
}
