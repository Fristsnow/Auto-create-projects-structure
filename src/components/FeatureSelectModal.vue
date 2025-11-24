<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NModal, NCard, NInput, NSpace, NGrid, NGi, NButton, NTag, NEmpty } from 'naive-ui'
import type { VueVersion, ProjectLang, FeatureKey, ComponentRegistryItem, ComponentRegistryPayload } from '@/types'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{ visible: boolean; value: FeatureKey[]; vueVersion: VueVersion; projectLang: ProjectLang }>()
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'update:value', v: FeatureKey[]): void }>()

const registry = ref<ComponentRegistryItem[]>([])
const query = ref('')

onMounted(async () => {
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    registry.value = v.components || []
  } catch (_) {}
})

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  const v3 = props.vueVersion === 'vue3'
  const v2 = props.vueVersion === 'vue2'
  const js = props.projectLang === 'js'
  return registry.value
    .filter(i => {
      const sup = i.supported
      if (sup) {
        if (v2 && sup.vue2 === false) return false
        if (v3 && sup.vue3 === false) return false
        if (js && sup.js === false) return false
        if (!js && sup.ts === false) return false
      }
      return true
    })
    .filter(i => !q || [i.key, i.label, i.package, ...(i.packages || []), i.desc].filter(Boolean).some(s => String(s).toLowerCase().includes(q)))
})

const selected = computed(() => new Set(props.value))
const selectedItems = computed(() => registry.value.filter(i => selected.value.has(i.key as FeatureKey)))
function toggle(key: FeatureKey) {
  const next = new Set(selected.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  emit('update:value', Array.from(next))
}

function close() { emit('update:visible', false) }
</script>

<template>
  <NModal :show="props.visible" preset="card" :style="{ width: '80vw', height: '80vh', margin: '0 auto' }" :mask-closable="true" @update:show="v => emit('update:visible', v)">
    <NCard title="选择组件" size="large" :bordered="true" class="modal-card">
      <div class="modal-shell">
        <!-- 顶部搜索（固定） -->
        <div class="search-bar">
          <NInput v-model:value="query" placeholder="搜索组件（名称/包名/描述）" clearable />
        </div>
        <!-- 下部内容（可滚动） -->
        <div class="content-area">
          <div class="left">
            <NGrid x-gap="12" y-gap="12" cols="1 900:2 1200:3">
              <NGi v-for="it in filtered" :key="it.key">
                <NCard :title="it.label || it.key" size="small" :bordered="true" :segmented="{ content: true }" class="big-item" @click="toggle(it.key as FeatureKey)">
                  <div class="desc">{{ it.desc }}</div>
                  <div class="meta">
                    <NTag size="small" type="info">{{ (it.packages?.[0] || it.package) || '未设置包名' }}</NTag>
                    <NTag size="small" :type="selected.has(it.key as FeatureKey) ? 'success' : 'default'">{{ selected.has(it.key as FeatureKey) ? '已选择' : '点击选择' }}</NTag>
                  </div>
                </NCard>
              </NGi>
            </NGrid>
          </div>
          <div class="right">
            <div v-if="selectedItems.length" class="mini-list">
              <NCard v-for="it in selectedItems" :key="it.key" size="small" :bordered="true" class="mini-item" @click="toggle(it.key as FeatureKey)">
                <div class="mini-title">{{ it.label || it.key }}</div>
                <div class="mini-state">{{ selected.has(it.key as FeatureKey) ? '已选择' : '未选择' }}</div>
              </NCard>
            </div>
            <div v-else class="mini-empty">
              <NEmpty description="尚未选择组件" />
            </div>
          </div>
        </div>
        <div class="actions">
          <NSpace justify="end">
            <NButton @click="close">关闭</NButton>
            <NButton type="primary" @click="close">完成选择（{{ props.value.length }}）</NButton>
          </NSpace>
        </div>
      </div>
    </NCard>
  </NModal>
</template>

<style scoped lang="scss">
.modal-card { height: 100%; }
.modal-shell { height: 100%; display: flex; flex-direction: column; }
.search-bar {
  position: sticky; top: 0; z-index: 2; background: #FFFFFF; border-bottom: 1px solid #CDDAE2;
  padding: 12px;
}
.content-area { flex: 1; overflow: auto; display: flex; gap: 12px; padding: 12px; }
.left { flex: 0 0 80%; min-width: 0; }
.right { flex: 0 0 20%; min-width: 220px; border-left: 1px solid #CDDAE2; padding-left: 12px; }
.mini-empty { display: flex; align-items: center; justify-content: center; height: 100%; }
.big-item { box-shadow: 0 8px 24px rgba(50, 89, 105, 0.06); }
.big-item .desc { color: #6C8FA9; margin-bottom: 8px; }
.big-item .meta { display: flex; gap: 8px; }
.mini-list { display: grid; grid-template-columns: 1fr; gap: 8px; }
.mini-item { display: flex; justify-content: space-between; align-items: center; }
.actions { padding: 12px; border-top: 1px solid #CDDAE2; background: #FFFFFF; }
</style>
