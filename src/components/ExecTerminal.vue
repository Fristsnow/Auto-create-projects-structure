<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { NModal, NButton } from 'naive-ui'

const props = defineProps<{ visible: boolean; logs: string[]; title?: string }>()
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void }>()

const scrollEl = ref<HTMLDivElement | null>(null)

watch(() => props.logs.length, async () => {
  await nextTick()
  const el = scrollEl.value
  if (el) {
    el.scrollTop = el.scrollHeight
  }
})
</script>

<template>
  <NModal :show="props.visible" :mask-closable="false" @update:show="v => emit('update:visible', v)">
    <div class="exec-terminal">
      <div class="header">
        <span>{{ props.title || '执行日志' }}</span>
        <NButton text @click="() => emit('update:visible', false)">关闭</NButton>
      </div>
      <div class="body" ref="scrollEl">
        <pre v-for="(line, i) in props.logs" :key="i" class="line">{{ line }}</pre>
      </div>
    </div>
  </NModal>
  
</template>

<style scoped lang="scss">
.exec-terminal {
  width: 800px;
  max-width: 90vw;
  border-radius: 8px;
  overflow: hidden;
  background: #0b0b0b; // 终端背景
  color: #e6e6e6;
  box-shadow: 0 6px 20px rgba(0,0,0,0.4);

  .header {
    padding: 10px 14px;
    font-weight: 600;
    border-bottom: 1px solid #1f1f1f;
    background: #111;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .body {
    max-height: 60vh;
    padding: 12px 14px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    line-height: 1.6;
    overflow: auto;
  }
  .line {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }
}
</style>