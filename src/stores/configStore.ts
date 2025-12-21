import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, MigrationResult } from '../types/config'

export const useConfigStore = defineStore('config', () => {
  // 状态
  const config = ref<AppConfig | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 获取配置
  const fetchConfig = async (): Promise<AppConfig | null> => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<AppConfig>('get_config')
      config.value = result
      return result
    } catch (e) {
      error.value = e as string
      console.error('获取配置失败:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // 保存配置（不执行迁移）
  const saveConfigOnly = async (newConfig: AppConfig): Promise<boolean> => {
    loading.value = true
    error.value = null

    try {
      await invoke('save_config_only', { config: newConfig })
      config.value = newConfig
      return true
    } catch (e) {
      error.value = e as string
      console.error('保存配置失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  // 执行数据迁移
  const migrateData = async (newConfig: AppConfig): Promise<MigrationResult | null> => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<MigrationResult>('migrate_data', { newConfig })

      if (result.success) {
        config.value = newConfig
      }

      return result
    } catch (e) {
      error.value = e as string
      console.error('数据迁移失败:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  return {
    config,
    loading,
    error,
    fetchConfig,
    saveConfigOnly,
    migrateData
  }
})
