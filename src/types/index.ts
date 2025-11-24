export type VueVersion = 'vue2' | 'vue3'
export type ProjectLang = 'ts' | 'js'

export interface EnvStatus {
  node?: string
  pnpm?: string
}

export type FeatureKey = 'router' | 'pinia' | 'sass' | 'naive-ui' | 'vfonts' | 'xicons' | string

export interface ComponentRegistryItem {
  key: string
  label: string
  package?: string
  packages?: string[]
  desc?: string
  version?: string
  versions?: { vue2?: string; vue3?: string }
  supported?: { vue2: boolean; vue3: boolean; ts: boolean; js: boolean }
  dev?: boolean
}

export interface ComponentRegistryPayload {
  components: ComponentRegistryItem[]
}

export interface ToolMenuItem {
  key: string
  label: string
  route?: string
  icon?: string
  enabled?: boolean
}

export interface ToolMenuPayload {
  tools: ToolMenuItem[]
  visible: boolean
}
