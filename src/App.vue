<script setup lang="ts">
import { NConfigProvider, NGlobalStyle, NMessageProvider, NDialogProvider, lightTheme } from 'naive-ui'
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'
import ComponentRegistryEditor from '@/components/ComponentRegistryEditor.vue'
</script>

<template>
  <NConfigProvider :theme="lightTheme">
    <NGlobalStyle />
    <NMessageProvider>
      <NDialogProvider>
        <ComponentRegistryEditor />
        <router-view />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
  
</template>

<style scoped>
:host {
  display: block;
}
</style>
