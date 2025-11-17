<script setup lang="ts">
import { NForm, NFormItem, NInput, NRadioGroup, NRadio, NButton, NSpace, NIcon } from 'naive-ui'
import { FolderOpenOutline } from '@vicons/ionicons5'
import { open } from '@tauri-apps/plugin-dialog'
import type { VueVersion, ProjectLang } from '@/types'

const props = defineProps<{ projectName: string; vueVersion: VueVersion; projectLang: ProjectLang; targetDir: string; setDefaultDir: boolean }>()
const emit = defineEmits<{
  (e: 'update:projectName', v: string): void
  (e: 'update:vueVersion', v: VueVersion): void
  (e: 'update:projectLang', v: ProjectLang): void
  (e: 'update:targetDir', v: string): void
  (e: 'update:setDefaultDir', v: boolean): void
  (e: 'openPathSelector'): void
}>()

async function pickDirectory() {
  const dir = await open({ directory: true, multiple: false })
  if (typeof dir === 'string') emit('update:targetDir', dir)
}
</script>

<template>
  <NForm label-placement="left" label-width="120">
    <NFormItem label="项目名称">
      <NInput :value="props.projectName" placeholder="请输入项目名称" @update:value="v => emit('update:projectName', v)" />
    </NFormItem>

    <NFormItem label="Vue 版本">
      <NRadioGroup :value="props.vueVersion" @update:value="v => emit('update:vueVersion', v as VueVersion)">
        <NSpace>
          <NRadio value="vue3">Vue 3（推荐）</NRadio>
          <NRadio value="vue2">Vue 2</NRadio>
        </NSpace>
      </NRadioGroup>
    </NFormItem>

    <NFormItem label="项目语言">
      <NRadioGroup :value="props.projectLang" @update:value="v => emit('update:projectLang', v as ProjectLang)">
        <NSpace>
          <NRadio value="ts">TypeScript</NRadio>
          <NRadio value="js">JavaScript</NRadio>
        </NSpace>
      </NRadioGroup>
    </NFormItem>

    <NFormItem label="存放目录">
      <div class="dir-row">
        <NInput :value="props.targetDir" placeholder="请选择或输入目录" @update:value="v => emit('update:targetDir', v)" />
<!--        <NButton secondary @click="pickDirectory">-->
<!--          <NIcon style="margin-right:6px"><FolderOpenOutline /></NIcon>选择-->
<!--        </NButton>-->
        <NButton tertiary @click="emit('openPathSelector')">选择</NButton>
      </div>
    </NFormItem>

    <NFormItem label="设为默认路径">
      <input type="checkbox" :checked="props.setDefaultDir" @change="e => emit('update:setDefaultDir', (e.target as HTMLInputElement).checked)" />
    </NFormItem>
  </NForm>
</template>

<style scoped lang="scss">
.dir-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>