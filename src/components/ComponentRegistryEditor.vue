<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { NButton, NForm, NFormItem, NInput, NCheckboxGroup, NCheckbox, NTable, NDrawer, NDrawerContent, useMessage, NSelect } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import type { ComponentRegistryItem, ComponentRegistryPayload } from '@/types'
const message = useMessage()

const list = ref<ComponentRegistryItem[]>([])
// 仅在抽屉中展示一个示例，不展示用户新增的条目
const demoRows = ref<ComponentRegistryItem[]>([
  {
    key: 'example-lib',
    label: '示例组件库',
    package: 'your-package-name',
    desc: '此处仅展示示例，新增组件不会在此列表显示',
    supported: { vue2: true, vue3: true, ts: true, js: true },
    dev: false,
    versions: { vue2: '^2.x', vue3: '^3.x' }
  }
])
const form = ref<ComponentRegistryItem>({ key: '', label: '', package: '', desc: '', supported: { vue2: true, vue3: true, ts: true, js: true }, dev: false, versions: { vue2: '', vue3: '' } })
const supportedValues = ref<string[]>(['vue2','vue3','ts','js'])
const show = ref(false)
const versionOptions = ref<{ label: string; value: string }[]>([
  { label: 'latest', value: 'latest' },
  { label: '^5.x', value: '^5.x' },
  { label: '^4.x', value: '^4.x' },
  { label: '^4.6.3', value: '^4.6.3' },
  { label: '^3.x', value: '^3.x' },
  { label: '^2.x', value: '^2.x' }
])

function syncSupportedValuesFromForm() {
  const s = form.value.supported || { vue2: true, vue3: true, ts: true, js: true }
  const vals: string[] = []
  if (s.vue2) vals.push('vue2')
  if (s.vue3) vals.push('vue3')
  if (s.ts) vals.push('ts')
  if (s.js) vals.push('js')
  supportedValues.value = vals
}

watch(supportedValues, (vals) => {
  form.value.supported = {
    vue2: vals.includes('vue2'),
    vue3: vals.includes('vue3'),
    ts: vals.includes('ts'),
    js: vals.includes('js')
  }
})

async function load() {
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    list.value = v.components || []
  } catch (_) { /* ignore */ }
}

async function save() {
  try {
    await invoke('save_component_registry', { payload: { components: list.value } })
    message.success('组件库已保存')
  } catch (e) {
    message.error(`保存失败：${String(e)}`)
  }
}

function addItem() {
  if (!form.value.key || !form.value.package) { message.warning('请填写 key 与 package'); return }
  const exists = list.value.find((i: ComponentRegistryItem) => i.key === form.value.key)
  if (exists) { message.warning('key 已存在'); return }
  list.value.push(JSON.parse(JSON.stringify(form.value)))
  form.value = { key: '', label: '', package: '', desc: '', supported: { vue2: true, vue3: true, ts: true, js: true }, dev: false, versions: { vue2: '', vue3: '' } }
  syncSupportedValuesFromForm()
  void save()
}

function removeItem(key: string) {
  list.value = list.value.filter((i: ComponentRegistryItem) => i.key !== key)
  void save()
}

onMounted(async () => {
  await load()
  syncSupportedValuesFromForm()
})
</script>

<template>
  <div class="registry-toolbar">
    <NButton size="small" secondary @click="show = true">组件库扩充</NButton>
  </div>
  <NDrawer v-model:show="show" placement="top" height="720" :trap-focus="false" :mask-closable="true" :z-index="3000">
    <NDrawerContent title="组件库扩充（持久化）">
    <div class="drawer-content-body">
    <NTable :bordered="false" :single-line="false" style="margin-bottom:12px">
      <thead>
        <tr>
          <th>Key</th>
          <th>包名</th>
          <th>描述</th>
          <th>支持</th>
          <th>版本(Vue2/Vue3)</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in demoRows" :key="item.key">
          <td>{{ item.key }}</td>
          <td><code>{{ item.package }}</code></td>
          <td>{{ item.desc }}</td>
          <td>
            <small>
              Vue2: {{ item.supported?.vue2 ? '✔' : '✘' }} · Vue3: {{ item.supported?.vue3 ? '✔' : '✘' }} ·
              TS: {{ item.supported?.ts ? '✔' : '✘' }} · JS: {{ item.supported?.js ? '✔' : '✘' }}
            </small>
          </td>
          <td>
            <small>
              <code>{{ item.versions?.vue2 || '-' }}</code> / <code>{{ item.versions?.vue3 || '-' }}</code>
            </small>
          </td>
        </tr>
        <tr>
          <td colspan="4">
            <small style="color:#666">提示：新增的组件将保存到注册库，用于新项目选择，但不会在此列表展示。</small>
          </td>
        </tr>
      </tbody>
    </NTable>

    <NForm label-placement="left" label-width="80px">
      <NFormItem label="Key"><NInput v-model:value="form.key" placeholder="唯一键，如 element-plus" /></NFormItem>
      <NFormItem label="包名"><NInput v-model:value="form.package" placeholder="如 element-plus 或 @unocss/reset" /></NFormItem>
      <NFormItem label="描述"><NInput v-model:value="form.desc" placeholder="简要说明" /></NFormItem>
      <NFormItem label="Vue2 版本">
        <NSelect v-model:value="form.versions!.vue2" :options="versionOptions" filterable tag clearable placeholder="输入或搜索版本" />
      </NFormItem>
      <NFormItem label="Vue3 版本">
        <NSelect v-model:value="form.versions!.vue3" :options="versionOptions" filterable tag clearable placeholder="输入或搜索版本" />
      </NFormItem>
      <NFormItem label="支持">
        <NCheckboxGroup v-model:value="supportedValues">
          <NCheckbox value="vue2">Vue2</NCheckbox>
          <NCheckbox value="vue3">Vue3</NCheckbox>
          <NCheckbox value="ts">TS</NCheckbox>
          <NCheckbox value="js">JS</NCheckbox>
        </NCheckboxGroup>
      </NFormItem>
      <NFormItem label="开发依赖"><NCheckbox v-model:checked="form.dev">作为 dev 依赖</NCheckbox></NFormItem>
      <NButton type="primary" @click="addItem">新增并保存</NButton>
    </NForm>
    </div>
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped lang="scss">
.registry-toolbar {
  position: fixed;
  top: 8px;
  right: 16px;
  z-index: 2000;
  display: flex;
  justify-content: flex-end;
}
code { color: #2f54eb }
.drawer-content-body { max-height: 700px; overflow: auto; }
</style>