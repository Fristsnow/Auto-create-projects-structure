<script setup lang="ts">
import { computed } from 'vue'
import { NSteps, NStep } from 'naive-ui'

const props = defineProps<{ missingEnv: { node: boolean; pnpm: boolean }; targetDir: string }>()
const current = computed(() => (props.missingEnv.node || props.missingEnv.pnpm ? 1 : (props.targetDir ? 3 : 2)))
</script>

<template>
  <div class="steps-panel">
    <NSteps size="small" :current="current" vertical>
      <NStep title="环境检查" description="检测 Node 与 pnpm" />
      <NStep title="填写信息" description="名称、版本、存放目录" />
      <NStep title="创建项目" description="生成并安装依赖" />
    </NSteps>
  </div>
</template>

<style scoped lang="scss">
.steps-panel {
  background: #FFFFFF;
  border: 1px solid #CDDAE2;
  box-shadow: 0 8px 24px rgba(50, 89, 105, 0.08);
  border-radius: 12px;
  padding: 12px;
}
</style>
