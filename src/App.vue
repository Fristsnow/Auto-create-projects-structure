<script setup lang="ts">
import { ref } from 'vue'
import {
  NConfigProvider,
  NGlobalStyle,
  NMessageProvider,
  NDialogProvider,
  NLayout,
  NLayoutHeader,
  NLayoutSider,
  NLayoutContent,
  NCard,
  NSpace,
  NGradientText,
  NButton,
  NIcon,
  lightTheme,
  type GlobalThemeOverrides,
} from 'naive-ui'
import { SettingsOutline } from '@vicons/ionicons5'
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'
import ComponentRegistryEditor from '@/components/ComponentRegistryEditor.vue'
import FloatingToolMenu from '@/components/FloatingToolMenu.vue'
import ConsolePanel from '@/components/ConsolePanel.vue'
import { useToolMenuStore } from '@/store/toolMenu'

const showRegistry = ref(true)
const showConsole = ref(false)
const menuStore = useToolMenuStore()
void menuStore.load()

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#2f54eb',
    primaryColorHover: '#6C8FA9',
    primaryColorPressed: '#1d37a3',
    primaryColorSuppl: '#6C8FA9',
    baseColor: '#FFFFFF',
    bodyColor: '#FFFFFF',
    cardColor: '#FFFFFF',
    modalColor: '#FFFFFF',
    popoverColor: '#FFFFFF',
    textColorBase: '#2f54eb',
    textColor1: '#2f54eb',
    textColor2: '#6C8FA9',
    textColor3: '#6C8FA9',
    borderColor: '#CDDAE2',
    dividerColor: '#CDDAE2'
  },
  Card: {
    color: '#FFFFFF',
    borderColor: '#CDDAE2',
    boxShadow: '0 8px 24px rgba(50, 89, 105, 0.08)'
  },
  Layout: {
    headerColor: 'rgba(255,255,255,0.72)',
    siderColor: 'transparent',
    textColor: '#2f54eb',
    borderColor: '#CDDAE2'
  },
  Button: {
    colorPrimary: '#2f54eb',
    colorHoverPrimary: '#6C8FA9',
    colorPressedPrimary: '#1d37a3',
    textColorPrimary: '#FFFFFF',
    borderPrimary: '#2f54eb'
  },
  Menu: {
    itemColorActive: 'rgba(108, 143, 169, 0.08)',
    itemColorHover: 'rgba(108, 143, 169, 0.12)',
    itemTextColorActive: '#2f54eb',
    itemTextColorHover: '#2f54eb',
    borderColorHorizontal: '#CDDAE2'
  },
  Tag: {
    colorInfo: 'rgba(108, 143, 169, .12)',
    textColorInfo: '#2f54eb',
    borderPrimary: '#CDDAE2'
  },
  Divider: {
    color: '#CDDAE2'
  },
  Input: {
    borderColor: '#CDDAE2',
    borderHover: '#6C8FA9',
    borderFocus: '#325969'
  },
  Select: {
    borderColor: '#CDDAE2'
  },
  Modal: {
    color: '#FFFFFF',
    borderRadius: '12px'
  }
}
</script>

<template>
  <NConfigProvider :theme="lightTheme" :theme-overrides="themeOverrides">
    <NGlobalStyle />
    <NMessageProvider>
      <NDialogProvider>
        <div class="app-shell">
          <NLayout>
            <NLayoutHeader class="app-header">
              <NSpace justify="space-between" align="center" style="width: 100%">
                <NGradientText gradient="linear-gradient(90deg, #6C8FA9 0%, #C8ADC4 100%)" size="20">
                  Auto Create Project Structure
                </NGradientText>
                <NSpace align="center">
                  <ComponentRegistryEditor inline />
                  <NButton tertiary size="small" @click="showConsole = true">
                    控制台
                  </NButton>
                </NSpace>
              </NSpace>
            </NLayoutHeader>

            <NLayout class="app-main" has-sider>
              <NLayoutSider class="app-sider" width="220" content-style="padding:0">
                <FloatingToolMenu />
              </NLayoutSider>
              <NLayoutContent class="app-content">
                <router-view />
                <ConsolePanel v-model:visible="showConsole" />
              </NLayoutContent>
            </NLayout>
          </NLayout>
        </div>
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
  
</template>

<style scoped lang="scss">
.app-shell {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-header {
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(4px);
  border-bottom: 1px solid #CDDAE2;
  position: sticky;
  top: 0;
  z-index: 1200;
  box-shadow: 0 4px 12px rgba(50, 89, 105, 0.06);
}

.app-main {
  flex: 1;
  height: calc(100vh - 56px);
  display: flex;
  overflow: hidden;
}

.app-content {
  padding: 16px;
  overflow: auto;
  flex: 1;
  height: 100%;
  background: linear-gradient(180deg, rgba(205,218,226,0.35) 0%, rgba(255,255,255,0.92) 100%), radial-gradient(700px 280px at 90% 18%, rgba(200,173,196,0.28) 0%, rgba(200,173,196,0) 60%);
}

.app-sider {
  background: #FFFFFF;
  border-right: 1px solid #CDDAE2;
  box-shadow: 6px 0 24px rgba(50, 89, 105, 0.04);
  padding: 0;
  height: 100%;
  overflow: hidden;
}
</style>
