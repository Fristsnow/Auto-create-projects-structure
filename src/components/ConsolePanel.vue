<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NModal, NCard, NSpace, NDivider, NSwitch, NButton } from 'naive-ui'
import { useToolMenuStore } from '@/store/toolMenu'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void }>()

const store = useToolMenuStore()

onMounted(() => { if (!store.tools.length) void store.load() })

function close() { emit('update:visible', false) }
</script>

<template>
  <NModal :show="props.visible" preset="card" :style="{ width: '80vw', height: '80vh', margin: '0 auto' }" :mask-closable="true" @update:show="v => emit('update:visible', v)">
    <NCard title="控制台" size="large" :bordered="true" style="height: 100%">
      <NSpace vertical>
        <NSpace align="center" justify="space-between">
          <span>显示右侧工具菜单</span>
          <NSwitch :value="store.visible" @update:value="store.setVisible" />
        </NSpace>
        <NDivider title-placement="left">菜单项</NDivider>
        <NSpace vertical>
          <NSpace v-for="item in store.tools" :key="item.key" align="center" justify="space-between">
            <span>{{ item.label }}</span>
            <NSwitch :value="item.enabled !== false" @update:value="(v) => store.toggle(item.key, v)" />
          </NSpace>
        </NSpace>
        <NSpace justify="end">
          <NButton type="primary" @click="close">关闭</NButton>
        </NSpace>
      </NSpace>
    </NCard>
  </NModal>
</template>

<style scoped lang="scss">
.n-modal { display: flex; align-items: center; }
</style>
