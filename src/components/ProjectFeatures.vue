<!--
 * @Author: FirstsnowLucky firstsnow1119@163.com
 * @Date: 2025-11-17 09:08:47
 * @LastEditors: FirstsnowLucky firstsnow1119@163.com
 * @LastEditTime: 2025-11-17 12:27:27
 * @FilePath: \Auto-create-projects-structure\src\components\ProjectFeatures.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { NCard, NTag, NGrid, NGi, NIcon, useDialog, useMessage, NModal, NForm, NFormItem, NSelect, NButton } from 'naive-ui'
import { GitBranchOutline, StorefrontOutline, ColorPaletteOutline, SparklesOutline, TextOutline, LayersOutline } from '@vicons/ionicons5'
import type { VueVersion, FeatureKey, ProjectLang, ComponentRegistryPayload, ComponentRegistryItem } from '@/types'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{ vueVersion: VueVersion; projectLang: ProjectLang; modelValue: FeatureKey[] }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: FeatureKey[]): void
}>()

interface FeatureMeta {
  key: FeatureKey
  name: string
  version: string
  desc: string
  icon: any
  disabled: boolean
  fromRegistry: boolean
}

const metas = computed<FeatureMeta[]>(() => {
  const v3 = props.vueVersion === 'vue3'
  const v2 = props.vueVersion === 'vue2'
  const js = props.projectLang === 'js'
  // 仅从注册库渲染所有特性
  const iconMap: Record<string, any> = {
    'router': GitBranchOutline,
    'pinia': StorefrontOutline,
    'sass': ColorPaletteOutline,
    'naive-ui': SparklesOutline,
    'vfonts': TextOutline,
    'xicons': LayersOutline
  }
  return registry.value.map((i) => ({
    key: i.key as FeatureKey,
    name: i.label || i.key,
    version: i.versions ? (props.vueVersion === 'vue3' ? (i.versions.vue3 || '') : (i.versions.vue2 || '')) : (i.version || ''),
    desc: i.desc || '',
    icon: iconMap[i.key] || LayersOutline,
    disabled: !!((v2 && i.supported && !i.supported.vue2) || (v3 && i.supported && !i.supported.vue3) || (js && i.supported && !i.supported.js) || (!js && i.supported && !i.supported.ts)),
    fromRegistry: true
  }))
})

// 注册库数据
const registry = ref<ComponentRegistryItem[]>([])
onMounted(async () => {
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    registry.value = v.components || []
  } catch (_) { /* ignore */ }
})

const selected = computed(() => new Set(props.modelValue))
function toggle(key: FeatureKey, disabled: boolean) {
  if (disabled) return
  const next = new Set(selected.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  emit('update:modelValue', Array.from(next))
}

const dialog = useDialog()
const message = useMessage()

function onRightClick(m: FeatureMeta) {
  if (!m.fromRegistry) { message.info('该项非注册库来源，不能编辑'); return }
  openVersionModal(m)
}

async function removeFromRegistry(key: FeatureKey) {
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    const arr = (v.components || []).filter((i: ComponentRegistryItem) => i.key !== key)
    await invoke('save_component_registry', { payload: { components: arr } })
    registry.value = arr
    message.success('已删除并更新注册库')
  } catch (e) {
    message.error(`删除失败：${String(e)}`)
  }
}

// 版本编辑
interface VersionForm { key: FeatureKey; vue2: string; vue3: string }
const showVersionModal = ref(false)
const versionForm = ref<VersionForm>({ key: '' as FeatureKey, vue2: '', vue3: '' })
const versionOptions = ref<{ label: string; value: string }[]>([
  { label: 'latest', value: 'latest' }
])

function openVersionModal(m: FeatureMeta) {
  const it = registry.value.find(i => i.key === m.key)
  versionForm.value = {
    key: m.key,
    vue2: it?.versions?.vue2 || '',
    vue3: it?.versions?.vue3 || ''
  }
  // 动态加载版本：优先使用 packages 的首个包名，否则使用 package
  const pkg = (it?.packages && it.packages[0]) || it?.package || ''
  versionOptions.value = [{ label: 'latest', value: 'latest' }]
  if (pkg) {
    invoke<string[]>('fetch_npm_versions', { package: pkg }).then(list => {
      const opts = list.map(v => ({ label: v, value: v }))
      // 合并现有值，保证可选
      const extras: string[] = [versionForm.value.vue2, versionForm.value.vue3, m.version].filter(Boolean)
      extras.forEach(v => { if (v && !list.includes(v)) { opts.unshift({ label: v, value: v }) } })
      versionOptions.value = [{ label: 'latest', value: 'latest' }, ...opts]
    }).catch(() => {
      // 回退：保留现有值
      const extras: string[] = [versionForm.value.vue2, versionForm.value.vue3, m.version].filter(Boolean)
      versionOptions.value = [{ label: 'latest', value: 'latest' }, ...extras.map(v => ({ label: v, value: v }))]
    })
  }
  showVersionModal.value = true
}

async function saveVersion() {
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    const arr = (v.components || []).map((i: ComponentRegistryItem) => {
      if (i.key === versionForm.value.key) {
        const next = { ...i }
        next.versions = next.versions || {}
        next.versions.vue2 = versionForm.value.vue2
        next.versions.vue3 = versionForm.value.vue3
        return next
      }
      return i
    })
    await invoke('save_component_registry', { payload: { components: arr } })
    registry.value = arr
    showVersionModal.value = false
    message.success('版本已更新并持久化')
  } catch (e) {
    message.error(`保存失败：${String(e)}`)
  }
}
</script>

<template>
  <NGrid x-gap="16" y-gap="16" cols="1 600:2 1000:3">
    <NGi v-for="m in metas" :key="m.key">
      <NCard size="medium" :class="['feature-card', selected.has(m.key) ? 'selected' : '', m.disabled ? 'disabled' : '']" :segmented="{ content: true }" @click="toggle(m.key, m.disabled)" @contextmenu.prevent="onRightClick(m)">
        <div class="header">
          <NIcon size="22"><component :is="m.icon" /></NIcon>
          <div class="title">{{ m.name }}</div>
          <NTag size="small" type="info">{{ m.version }}</NTag>
        </div>
        <div class="desc">{{ m.desc }}</div>
        <div class="footer">
          <NTag v-if="m.disabled" size="small" type="warning">不支持当前版本</NTag>
          <NTag v-else size="small" type="success">{{ selected.has(m.key) ? '已选择' : '可选择' }}</NTag>
        </div>
      </NCard>
    </NGi>
  </NGrid>

  <!-- 版本编辑弹窗 -->
  <NModal v-model:show="showVersionModal">
    <NCard class="version-modal-card" title="菜单选项" size="small">
      <div class="version-modal-body">
        <NForm label-placement="left" label-width="84">
          <div class="version-row">
            <NFormItem label="Vue 2 版本">
              <NSelect v-model:value="versionForm.vue2" :options="versionOptions" filterable tag clearable placeholder="输入或搜索版本" />
            </NFormItem>
            <NFormItem label="Vue 3 版本">
              <NSelect v-model:value="versionForm.vue3" :options="versionOptions" filterable tag clearable placeholder="输入或搜索版本" />
            </NFormItem>
          </div>
        </NForm>
      </div>
      <template #action>
        <div class="version-modal-actions">
          <NButton type="primary" size="small" @click="saveVersion">保存</NButton>
          <NButton size="small" @click="showVersionModal = false">取消</NButton>
          <NButton tertiary type="error" size="small" @click="removeFromRegistry(versionForm.key)">删除该项</NButton>
        </div>
      </template>
    </NCard>
  </NModal>
</template>

 

<style scoped lang="scss">
.feature-card {
  cursor: pointer;
  transition: border-color .2s, box-shadow .2s, transform .1s;
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .title {
    font-weight: 600;
    flex: 1;
  }
  .desc {
    color: #666;
    font-size: 13px;
  }
  .footer {
    margin-top: 8px;
  }
}
.feature-card.selected {
  border-color: #2f54eb;
  box-shadow: 0 0 0 2px rgba(47, 84, 235, .15);
}
.feature-card.disabled {
  opacity: .6;
  cursor: not-allowed;
}

// 版本弹窗样式（Sass）
.version-modal-card {
  max-width: 520px;
  width: 92vw;
  margin: 0 auto;
}
.version-modal-body {
  max-height: 50vh;
  overflow: auto;
}
.version-modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
.version-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
</style>