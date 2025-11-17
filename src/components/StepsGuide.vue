<script setup lang="ts">
import { computed } from 'vue'
import { NSteps, NStep } from 'naive-ui'

const props = defineProps<{ missingEnv: { node: boolean; pnpm: boolean }; targetDir: string }>()
const current = computed(() => (props.missingEnv.node || props.missingEnv.pnpm ? 1 : (props.targetDir ? 3 : 2)))
</script>

<template>
  <NSteps size="small" :current="current" style="margin-bottom: 4px;">
    <NStep title="环境检查" description="检测 Node 与 pnpm" />
    <NStep title="填写信息" description="名称、版本、存放目录" />
    <NStep title="创建项目" description="生成并安装依赖" />
  </NSteps>
</template>