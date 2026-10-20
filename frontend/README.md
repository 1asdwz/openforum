# frontend

The Vue 3 single-page application of OpenForum. See the root documentation:

- [README](../README.md) / [简体中文](../README.zh-CN.md)
- [Local development](../docs/DEVELOPMENT.md)
- [Deployment](../docs/DEPLOYMENT.md)

```bash
npm install
npm run dev        # http://localhost:5173/forum/
npm run type-check
npm run build
```

Development variables live in `.env.development`, production in `.env.production`. The build base path is controlled by `VITE_BASE_PATH` (default `/forum/`).
