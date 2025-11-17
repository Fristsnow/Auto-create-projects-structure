/*
 * @Author: FirstsnowLucky firstsnow1119@163.com
 * @Date: 2025-11-17 08:37:23
 * @LastEditors: FirstsnowLucky firstsnow1119@163.com
 * @LastEditTime: 2025-11-17 08:45:54
 * @FilePath: \Auto-create-projects-structure\src\router\index.ts
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import CreateProject from '@/views/CreateProject.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'CreateProject',
    component: CreateProject,
  },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})