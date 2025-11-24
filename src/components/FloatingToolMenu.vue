<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NCard, NMenu } from 'naive-ui'
import { useToolMenuStore } from '@/store/toolMenu'

const router = useRouter()
const route = useRoute()
const store = useToolMenuStore()

const tools = computed(() => store.tools.filter(t => t.enabled !== false))

const options = computed(() => tools.value.map(t => ({ label: t.label, key: t.key })))
const activeKey = computed(() => {
  const p = route.path
  const seg = p.startsWith('/tool/') ? p.split('/')[2] : 'vue'
  return seg
})

function onSelect(key: string) {
  const item = tools.value.find(t => t.key === key)
  const path = item?.route || `/tool/${key}`
  router.push(path)
}

watch(() => route.path, () => { /* 触发重渲染以同步选中项 */ })
</script>

<template>
  <div v-if="store.visible" class="sider-menu">
    <NCard size="small" class="menu-card" :bordered="true" title="工具菜单">
      <div class="menu-content">
        <NMenu :value="activeKey" :options="options" @update:value="onSelect" />
      </div>
    </NCard>
  </div>
</template>

<style scoped lang="scss">
.sider-menu { padding: 12px 8px; height: 100%; }
.menu-card {
  height: 100%;
  background: linear-gradient(180deg, #FFFFFF 0%, rgba(205,218,226,0.4) 100%);
  border: 1px solid #CDDAE2;
  box-shadow: 0 8px 24px rgba(50, 89, 105, 0.08);
}
.menu-content {
  height: calc(100% - 36px);
  overflow-y: auto;
  padding: 8px 4px;
  /* 隐藏滚动条但允许滚动 */
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/Edge */
}
.menu-content::-webkit-scrollbar { width: 0; height: 0; }
:deep(.n-menu-item) { border-radius: 8px; }
:deep(.n-card__content) { height: 100%; }
</style>
