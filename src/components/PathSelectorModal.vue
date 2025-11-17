<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NModal, NCard, NSpace, NRadioGroup, NRadioButton, NInput, NButton, NDivider, NIcon, NSwitch, NAlert, NText } from 'naive-ui'
import { FolderOpenOutline, InformationCircleOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{
  (e: 'update:show', v: boolean): void
  (e: 'confirm', payload: { type: 'default' | 'custom'; path: string; remember: boolean }): void
}>()

const type = ref<'default' | 'custom'>('custom')
const path = ref('')
const remember = ref(false)
const selecting = ref(false)
const systemPaths = ref<{ desktop?: string; downloads?: string; documents?: string; pictures?: string }>({})
const error = ref('')

async function pickDirectory() {
  try {
    selecting.value = true
    error.value = ''
    const dir = await open({ directory: true, multiple: false, title: '选择路径' })
    if (typeof dir === 'string') path.value = dir
  } catch (e) {
    error.value = `选择路径失败：${String(e)}`
  } finally {
    selecting.value = false
  }
}

function quickPick(p?: string) {
  if (!p) return
  path.value = p
}

async function loadSystemPaths() {
  try {
    const res = await invoke<any>('get_system_paths')
    systemPaths.value = res || {}
  } catch (_) {
    // 非 Tauri 或命令缺失时忽略
  }
}

function onConfirm() {
  emit('confirm', { type: type.value, path: path.value, remember: remember.value })
}

function close() { emit('update:show', false) }

onMounted(loadSystemPaths)

const disabledConfirm = computed(() => type.value === 'custom' && !path.value)
</script>

<template>
  <NModal :show="props.show" preset="card" title="选择存放路径" @update:show="v => emit('update:show', v)">
    <div class="modal-body">
      <NText depth="3">为你的项目选择一个清晰的存放目录，支持默认或自定义路径。</NText>

      <div class="section">
        <NRadioGroup v-model:value="type">
          <NSpace>
            <!-- <NRadioButton value="default">默认路径</NRadioButton> -->
            <NRadioButton value="custom">自定义路径</NRadioButton>
          </NSpace>
        </NRadioGroup>
      </div>

      <div v-if="type === 'custom'" class="card">
        <div class="row">
          <NInput v-model:value="path" placeholder="请选择或输入路径">
            <template #prefix>
              <NIcon size="16"><FolderOpenOutline /></NIcon>
            </template>
          </NInput>
          <NButton secondary :loading="selecting" @click="pickDirectory">
            <NIcon style="margin-right:6px"><FolderOpenOutline /></NIcon>选择
          </NButton>
        </div>

        <NDivider style="margin: 12px 0" />

        <div class="quick">
          <div class="label">
            <NIcon style="margin-right:6px"><InformationCircleOutline /></NIcon>
            <NText depth="3">常用路径</NText>
          </div>
          <NSpace wrap size="small">
            <NButton quaternary size="small" @click="quickPick(systemPaths.desktop)">桌面</NButton>
            <NButton quaternary size="small" @click="quickPick(systemPaths.downloads)">下载</NButton>
            <NButton quaternary size="small" @click="quickPick(systemPaths.documents)">文档</NButton>
            <NButton quaternary size="small" @click="quickPick(systemPaths.pictures)">图片</NButton>
          </NSpace>
        </div>
      </div>

      <div class="row section">
        <NText depth="3">记住选择为默认路径</NText>
        <NSwitch v-model:value="remember" />
      </div>

      <NAlert v-if="error" type="error" :title="error" style="margin-top: 8px" />

      <div class="actions">
        <NButton @click="close">取消</NButton>
        <NButton type="primary" :disabled="disabledConfirm" @click="onConfirm">确定</NButton>
      </div>
    </div>
  </NModal>
  
</template>

<style scoped lang="scss">
.modal-body { display: flex; flex-direction: column; gap: 12px; }
.section { margin-top: 6px; }
.card { padding: 12px; border: 1px solid #e5e7eb; border-radius: 8px; background: #fafafa; }
.row { display: flex; gap: 8px; align-items: center; }
.quick { display: flex; align-items: center; justify-content: space-between; }
.label { display: flex; align-items: center; color: #666; font-size: 13px; }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
</style>