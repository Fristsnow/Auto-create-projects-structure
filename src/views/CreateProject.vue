<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NPageHeader, NCard, NSpace, NDivider, NGradientText, NButton, NIcon, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { PlayCircleOutline } from '@vicons/ionicons5'
import EnvStatus from '@/components/EnvStatus.vue'
import ProjectForm from '@/components/ProjectForm.vue'
import ProjectFeatures from '@/components/ProjectFeatures.vue'
import FeatureSelectModal from '@/components/FeatureSelectModal.vue'
import StepsGuide from '@/components/StepsGuide.vue'
import PathSelectorModal from '@/components/PathSelectorModal.vue'
import ExecTerminal from '@/components/ExecTerminal.vue'
import ComponentRegistryEditor from '@/components/ComponentRegistryEditor.vue'
import type { EnvStatus as EnvStatusType, VueVersion, FeatureKey, ProjectLang } from '@/types'

const projectName = ref('my-vue-app')
const vueVersion = ref<VueVersion>('vue3')
const projectLang = ref<ProjectLang>('ts')
const targetDir = ref('')
const setDefaultDir = ref(true)
const creating = ref(false)
const message = useMessage()
const features = ref<FeatureKey[]>(['sass', 'vfonts'])
const showFeatureModal = ref(false)
const registry = ref<ComponentRegistryItem[]>([])
const selectedNames = computed(() => {
  const map = new Map<string, string>()
  registry.value.forEach(i => map.set(i.key, i.label || i.key))
  return features.value.map(k => map.get(k) || k)
})

// 执行框状态
const showExec = ref(false)
const execLogs = ref<string[]>([])
function appendLog(s: string) { execLogs.value.push(s) }

const canCreate = computed(() => projectName.value && targetDir.value)

const env = ref<EnvStatusType>({})
const missingEnv = computed(() => ({
  node: !env.value.node,
  pnpm: !env.value.pnpm,
}))

async function checkEnv() {
  env.value = await invoke<EnvStatus>('check_environment')
}

async function createProject() {
  await checkEnv()
  if (missingEnv.value.node || missingEnv.value.pnpm) {
    message.warning('请先安装 Node 与 pnpm，再创建项目')
    return
  }
  try {
    let ok = false
    // 预检查：目标文件夹是否非空，若非空直接终止并提示
    try {
      await invoke('check_target_dir', { directory: targetDir.value, name: projectName.value })
    } catch (e) {
      message.error(String(e))
      return
    }
    // 打开执行框，初始化日志
    execLogs.value = []
    showExec.value = true
    appendLog(`开始创建项目：${projectName.value}`)
    appendLog(`目标目录：${targetDir.value}`)
    appendLog(`Vue 版本：${vueVersion.value}，语言：${projectLang.value}`)
    appendLog(`选择特性：${features.value.join(', ') || '无'}`)
    creating.value = true

    // 订阅日志与完成事件（先订阅，避免丢失早期日志）
    const unlistenLog = await listen<{ line: string }>('project:create_log', (e) => {
      const p: any = e.payload
      if (p?.line) appendLog(String(p.line))
    })
    const unlistenDone = await listen<{ success: boolean; error?: string }>('project:create_done', async (e) => {
      const p: any = e.payload
      if (p?.success) {
        ok = true
        appendLog('项目创建成功。')
        message.success('项目创建成功，已自动安装依赖')
      } else {
        const msg = String(p?.error || '未知错误')
        appendLog(`创建失败：${msg}`)
        message.error(`创建失败：${msg}`)
      }
      creating.value = false
      await unlistenLog()
      await unlistenDone()
      appendLog('执行结束。')
      setTimeout(() => {
        if (ok) showExec.value = false
      }, 1200)
    })

    // 调用后端异步创建命令（非阻塞）
    await invoke('create_project_async', {
      version: vueVersion.value,
      lang: projectLang.value,
      name: projectName.value,
      directory: targetDir.value,
      setDefault: setDefaultDir.value,
      features: features.value,
    })
  } catch (e) {
    appendLog(`创建失败：${String(e)}`)
    message.error(`创建失败：${String(e)}`)
  } finally {
    // 最终状态由完成事件处理；此处不再主动关闭
  }
}

async function initDefaultDir() {
  try {
    const d = await invoke<string | null>('read_default_directory')
    if (d) targetDir.value = d
  } catch (_) {
    // ignore
  }
}

onMounted(async () => {
  try {
    await checkEnv()
  } catch (_) {
    // 非 Tauri 环境忽略
  }
  await initDefaultDir()
  try {
    const v = await invoke<ComponentRegistryPayload>('read_component_registry')
    registry.value = v.components || []
  } catch (_) {}
})

const showPathSelector = ref(false)
function onPathConfirm(payload: { type: 'default' | 'custom'; path: string; remember: boolean }) {
  if (payload.type === 'custom' && payload.path) {
    targetDir.value = payload.path
  }
  setDefaultDir.value = payload.remember
  showPathSelector.value = false
}
</script>

<template>
  <div class="page app-container">
    <NPageHeader class="page-hero">
      <template #title>
        <NGradientText gradient="linear-gradient(90deg, #6C8FA9 0%, #C8ADC4 100%)" style="font-size:32px;">
          可视化创建 Vue 项目
        </NGradientText>
      </template>
      <template #subtitle>
        支持 Vue 2 与 Vue 3 · 简洁、直观、快速上手
      </template>
      
    </NPageHeader>

    <div class="fixed-steps">
      <StepsGuide :missing-env="missingEnv" :target-dir="targetDir" />
    </div>

    <NSpace vertical size="large">
      

      <NCard size="large" :segmented="{ content: true, footer: 'soft' }" title="运行环境">
        <EnvStatus :env="env" :missing-env="missingEnv" />
      </NCard>

      <NCard size="large" :segmented="{ content: true, footer: 'soft' }" title="项目信息">
        <ProjectForm
          :project-name="projectName"
          :vue-version="vueVersion"
          :project-lang="projectLang"
          :target-dir="targetDir"
          :set-default-dir="setDefaultDir"
          @update:projectName="v => projectName = v"
          @update:vueVersion="v => vueVersion = v"
          @update:projectLang="v => projectLang = v"
          @update:targetDir="v => targetDir = v"
          @update:setDefaultDir="v => setDefaultDir = v"
          @openPathSelector="showPathSelector = true"
        />

        <NDivider />

        <NSpace vertical>
          <NSpace align="center" justify="space-between">
            <NButton tertiary size="large" @click="showFeatureModal = true">选择组件</NButton>
            <div class="selected-count">已选：{{ features.length }} 项</div>
          </NSpace>
          <div class="selected-list">
            <div class="selected-grid">
              <NTag v-for="name in selectedNames" :key="name" size="small" type="success">{{ name }}</NTag>
            </div>
          </div>
        </NSpace>

        <NDivider />

        <NButton type="primary" size="large" :disabled="!canCreate || creating" :loading="creating" @click="createProject">
          <NIcon style="margin-right:6px"><PlayCircleOutline /></NIcon>创建项目
        </NButton>
      </NCard>

      <PathSelectorModal v-model:show="showPathSelector" @confirm="onPathConfirm" />
      <FeatureSelectModal v-model:visible="showFeatureModal" :vue-version="vueVersion" :project-lang="projectLang" v-model:value="features" />
      <ExecTerminal v-model:visible="showExec" :logs="execLogs" title="创建项目执行过程" />
    </NSpace>
  </div>
  
</template>

<style lang="scss" scoped>
.page {
  padding: 24px 16px 32px;
}
.page-hero { text-align: center; }
.page-hero :deep(.n-page-header__content) { justify-content: center; }
.page-hero :deep(.n-page-header__title) { display: flex; justify-content: center; }
.page-hero :deep(.n-page-header__subtitle) { text-align: center; color: #6C8FA9; }
.fixed-steps {
  position: fixed;
  right: 16px;
  top: 50%;
  transform: translateY(-50%);
  width: 280px;
  z-index: 1100;
}
.selected-count { color: #6C8FA9; }
.selected-list { padding-top: 8px; }
.selected-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
  max-height: 120px;
  overflow: auto;
  padding: 4px 0;
}
.feature-card {
  will-change: transform;
  transition: transform .12s ease-out, box-shadow .18s ease-out, border-color .18s ease-out;
}
.feature-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 24px rgba(50, 89, 105, 0.08);
}
.feature-card.selected {
  border-color: #325969;
  box-shadow: 0 0 0 2px rgba(50, 89, 105, .15);
}
</style>
