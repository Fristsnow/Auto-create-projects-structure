import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { ToolMenuItem, ToolMenuPayload } from '@/types'

export const useToolMenuStore = defineStore('toolMenu', {
  state: () => ({
    tools: [] as ToolMenuItem[],
    visible: true,
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true
      try {
        const payload = await invoke<ToolMenuPayload>('read_tool_menu')
        this.tools = payload.tools || []
        this.visible = payload.visible ?? true
      } catch (_) {
        // 默认数据
        this.tools = [
          { key: 'vue', label: 'Vue', route: '/tool/vue', enabled: true },
          { key: 'java', label: 'Java', route: '/tool/java', enabled: true },
          { key: 'node', label: 'Node', route: '/tool/node', enabled: true },
          { key: 'laravel', label: 'Laravel', route: '/tool/laravel', enabled: true },
        ]
        this.visible = true
      } finally {
        this.loading = false
      }
    },
    async save() {
      const payload: ToolMenuPayload = { tools: this.tools, visible: this.visible }
      try { await invoke('save_tool_menu', { payload }) } catch (_) {}
    },
    setVisible(v: boolean) {
      this.visible = v
      void this.save()
    },
    toggle(key: string, v: boolean) {
      const t = this.tools.find(i => i.key === key)
      if (t) { t.enabled = v; void this.save() }
    }
  }
})
