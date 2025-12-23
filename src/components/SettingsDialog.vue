<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useConfigStore } from '../stores/configStore'
import type { AppConfig } from '../types/config'

// Props
defineProps<{
  show: boolean
}>()

// Emits
const emit = defineEmits<{
  close: []
}>()

// Store
const configStore = useConfigStore()

// 状态
const rootDir = ref('')
const isMigrating = ref(false)
const migrationMessage = ref('')
const showResult = ref(false)

// 加载配置
onMounted(async () => {
  const config = await configStore.fetchConfig()
  if (config) {
    rootDir.value = config.rootDir
  }
})

// 选择根目录
const selectRootDir = async () => {
  const selected = await open({
    directory: true,
    multiple: false
  })

  if (selected) {
    rootDir.value = selected as string
  }
}

// 保存配置并迁移
const saveAndMigrate = async () => {
  if (!rootDir.value) {
    alert('请选择数据目录')
    return
  }

  isMigrating.value = true
  migrationMessage.value = '正在迁移数据，请稍候...'
  showResult.value = false

  const newConfig: AppConfig = {
    rootDir: rootDir.value
  }

  const result = await configStore.migrateData(newConfig)

  isMigrating.value = false

  if (result) {
    if (result.success) {
      migrationMessage.value = result.message
      showResult.value = true
    } else {
      migrationMessage.value = `迁移失败: ${result.message}`
      showResult.value = true
    }
  } else {
    migrationMessage.value = `迁移失败: ${configStore.error || '未知错误'}`
    showResult.value = true
  }
}

// 关闭对话框
const closeDialog = () => {
  emit('close')
}
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click.self="closeDialog">
    <div class="dialog-content">
      <div class="dialog-header">
        <h2>设置</h2>
        <button class="close-btn" @click="closeDialog">×</button>
      </div>

      <div class="dialog-body">
        <!-- 根目录 -->
        <div class="form-group">
          <label>数据目录：</label>
          <div class="input-group">
            <input v-model="rootDir" type="text" readonly />
            <button @click="selectRootDir">选择</button>
          </div>
          <p class="hint">数据库和图片将保存在此目录下</p>
        </div>

        <!-- 迁移提示 -->
        <div class="info-box">
          <p>⚠️ 修改目录后将自动执行数据迁移：</p>
          <ul>
            <li>复制整个数据目录到新位置</li>
            <li>包括数据库文件和所有图片</li>
            <li>旧目录的文件将保留，不会删除</li>
          </ul>
        </div>

        <!-- 迁移进度 -->
        <div v-if="isMigrating" class="migrating">
          <div class="spinner"></div>
          <p>{{ migrationMessage }}</p>
        </div>

        <!-- 迁移结果 -->
        <div v-if="showResult && !isMigrating" class="result-box">
          <p>{{ migrationMessage }}</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="closeDialog">取消</button>
        <button class="btn-save" @click="saveAndMigrate" :disabled="isMigrating">
          {{ isMigrating ? '迁移中...' : '保存并迁移' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-content {
  background-color: rgba(26, 26, 46, 0.95);
  border-radius: 8px;
  width: 600px;
  max-width: 90%;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-header h2 {
  color: white;
  font-size: 18px;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #ff4444;
}

.dialog-body {
  padding: 20px;
  max-height: 60vh;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  color: white;
  margin-bottom: 8px;
  font-size: 14px;
}

.hint {
  margin-top: 6px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
}

.input-group {
  display: flex;
  gap: 8px;
}

.input-group input {
  flex: 1;
  padding: 8px 12px;
  background-color: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  color: white;
  font-size: 14px;
}

.input-group button {
  padding: 8px 16px;
  background-color: rgba(100, 150, 255, 0.8);
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.input-group button:hover {
  background-color: rgba(100, 150, 255, 1);
}

.info-box {
  background-color: rgba(255, 200, 0, 0.1);
  border: 1px solid rgba(255, 200, 0, 0.3);
  border-radius: 4px;
  padding: 12px;
  margin-top: 20px;
}

.info-box p {
  color: #ffc800;
  margin: 0 0 8px 0;
  font-size: 14px;
}

.info-box ul {
  margin: 0;
  padding-left: 20px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 13px;
}

.info-box li {
  margin-bottom: 4px;
}

.migrating {
  text-align: center;
  padding: 20px;
}

.spinner {
  width: 40px;
  height: 40px;
  margin: 0 auto 12px;
  border: 4px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.migrating p {
  color: white;
  font-size: 14px;
}

.result-box {
  background-color: rgba(100, 200, 100, 0.1);
  border: 1px solid rgba(100, 200, 100, 0.3);
  border-radius: 4px;
  padding: 12px;
  margin-top: 20px;
}

.result-box p {
  color: #64c864;
  margin: 0;
  font-size: 14px;
  white-space: pre-line;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-footer button {
  padding: 10px 24px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel {
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
}

.btn-cancel:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

.btn-save {
  background-color: rgba(100, 150, 255, 0.8);
  color: white;
}

.btn-save:hover:not(:disabled) {
  background-color: rgba(100, 150, 255, 1);
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
