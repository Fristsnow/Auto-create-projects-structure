/*
 * @Author: FirstsnowLucky firstsnow1119@163.com
 * @Date: 2025-11-14 13:58:42
 * @LastEditors: FirstsnowLucky firstsnow1119@163.com
 * @LastEditTime: 2025-11-17 11:31:04
 * @FilePath: \Auto-create-projects-structure\src\main.ts
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
import { createApp } from 'vue'
// import './style.css'
import '@/styles/main.scss'
import App from './App.vue'
import { router } from '@/router'
import { pinia } from '@/store'

const app = createApp(App)

app.use(pinia)
app.use(router)

app.mount('#app')
