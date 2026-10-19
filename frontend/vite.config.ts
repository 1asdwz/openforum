import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import { viteStaticCopy } from 'vite-plugin-static-copy'
import { fileURLToPath, URL } from 'node:url'

// App deployment base path: defaults to /forum/ (apprest.mytruststores.com/forum);
// set VITE_BASE_PATH=/ to deploy at the root of a standalone domain (e.g. openforum.yingrest.com/).
export default defineConfig({
  base: process.env.VITE_BASE_PATH || '/forum/',
  plugins: [
    vue(),
    // Element Plus on-demand import: el-* components used in templates are auto-registered along with their styles
    Components({
      resolvers: [ElementPlusResolver()],
      dts: 'src/components.d.ts',
    }),
    // Copy the MediaPipe wasm runtime to dist/mediapipe so it can be loaded from /forum/mediapipe at runtime
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
      // Development: proxy /forum/api to the local backend, stripping the /forum prefix
      '/forum/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/forum/, ''),
      },
    },
  },
  build: {
    outDir: 'dist',
    // Put static assets in a separate content-hashed directory to enable long-term caching in nginx/Cloudflare
    assetsDir: 'static',
    chunkSizeWarningLimit: 1500,
  },
})
