import { createApp } from 'vue'
import { createPinia } from 'pinia'
// Element Plus 按需引入：命令式调用的组件需显式引入样式，
// 模板中的 el-* 组件样式由 unplugin-vue-components 自动注入
import 'element-plus/es/components/message/style/css'
import 'element-plus/es/components/message-box/style/css'

import App from './App.vue'
import router from './router'
import { useAuthStore } from './stores/auth'
import './styles/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

// 启动时恢复登录态（异步，不阻塞挂载）
useAuthStore().bootstrap()

app.mount('#app')
