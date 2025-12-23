// 窗口尺寸配置接口
export interface WindowSizeConfig {
  width: number
  height: number
}

// 应用配置接口
export interface AppConfig {
  rootDir: string
  mainWindow?: WindowSizeConfig
  imageViewerWindow?: WindowSizeConfig
  allTodosWindow?: WindowSizeConfig
}

// 迁移结果接口
export interface MigrationResult {
  success: boolean
  message: string
  databaseCopied: boolean
  imagesCopied: number
  pathsUpdated: number
}
