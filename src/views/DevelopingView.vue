<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { NCard } from 'naive-ui'

const route = useRoute()
const key = computed(() => String(route.params.key || ''))
</script>

<template>
  <NCard title="功能开发中" :bordered="true">
    <div style="padding:12px">
      <p>当前页面：{{ key }}</p>
      <p>左侧显示内容：开发中</p>
      <p>仅 Vue 提供创建页面，其它如 Java、Node、Laravel 进入此占位页。</p>
    </div>
  </NCard>
</template>

<style scoped lang="scss"></style>
