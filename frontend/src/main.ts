import { createApp } from 'vue'
import { createPinia } from 'pinia'
// Element Plus on-demand import: imperatively called components need their styles imported explicitly,
// while styles for el-* components in templates are injected automatically by unplugin-vue-components
import 'element-plus/es/components/message/style/css'
import 'element-plus/es/components/message-box/style/css'
// Dark mode variables (used with html.dark)
import 'element-plus/theme-chalk/dark/css-vars.css'
// Variable grotesk used by the premium community page
import '@fontsource-variable/plus-jakarta-sans'

import App from './App.vue'
import router from './router'
import { reveal } from './directives/reveal'
import { useAuthStore } from './stores/auth'
import { useUiStore } from './stores/ui'
import './styles/main.css'

const app = createApp(App)

app.directive('reveal', reveal)
app.use(createPinia())
app.use(router)

// Apply the theme on startup (synchronously, to avoid a first-paint flash)
useUiStore().init()

// Restore login state on startup (async, does not block mounting)
useAuthStore().bootstrap()

app.mount('#app')
