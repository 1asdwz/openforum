import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { viteStaticCopy } from 'vite-plugin-static-copy'
import { fileURLToPath, URL } from 'node:url'

// 应用部署在 https://apprest.mytruststores.com/forum/
export default defineConfig({
  base: '/forum/',
  plugins: [
    vue(),
    // 将 MediaPipe 的 wasm 运行时复制到 dist/mediapipe，运行时可从 /forum/mediapipe 加载
    viteStaticCopy({
      targets: [
        {
          src: 'node_modules/@mediapipe/tasks-vision/wasm/*',
          dest: 'mediapipe',
        },
      ],
    }),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 5173,
    proxy: {
      // 开发环境：将 /forum/api 代理到本地后端，并去掉 /forum 前缀
      '/forum/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/forum/, ''),
      },
    },
  },
  build: {
    outDir: 'dist',
    // 静态资源放在带内容哈希的独立目录，便于 nginx/Cloudflare 设置长期缓存
    assetsDir: 'static',
    chunkSizeWarningLimit: 1500,
  },
})
