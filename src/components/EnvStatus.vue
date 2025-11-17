<script setup lang="ts">
import { NTag, NAlert } from 'naive-ui'
import type { EnvStatus } from '@/types'

defineProps<{ env: EnvStatus; missingEnv: { node: boolean; pnpm: boolean } }>()
</script>

<template>
  <div>
    <div class="env-row">
      <NTag type="success" v-if="env.node">Node: {{ env.node }}</NTag>
      <NTag type="error" v-else>未检测到 Node</NTag>
      <NTag type="success" v-if="env.pnpm">pnpm: {{ env.pnpm }}</NTag>
      <NTag type="error" v-else>未检测到 pnpm</NTag>
    </div>
    <NAlert v-if="missingEnv.node || missingEnv.pnpm" type="warning" title="环境缺失" style="margin-top:12px">
      请安装必要环境：
      <a href="https://nodejs.org/en" target="_blank">Node.js</a>
      ，
      <a href="https://pnpm.io/installation" target="_blank">pnpm</a>
      。
    </NAlert>
  </div>
</template>

<style scoped lang="scss">
.env-row {
  display: flex;
  gap: 12px;
  align-items: center;
}
</style>