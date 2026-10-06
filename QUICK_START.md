## OpenForum 快速启动指南

### 一键启动（推荐）

#### Linux / macOS

```bash
# 给脚本执行权限
chmod +x start.sh stop.sh

# 启动所有服务
bash start.sh

# 停止所有服务
bash stop.sh
```

#### Windows (PowerShell)

```powershell
# 启动
docker-compose up -d

# 停止
docker-compose down
```

### 手动启动

#### 1. 启动 MySQL 和 Redis

```bash
docker-compose up -d mysql redis
```

#### 2. 启动后端

```bash
cd backend
mvn spring-boot:run
```

或使用 Docker：

```bash
docker-compose up -d backend
```

#### 3. 启动前端

```bash
cd frontend
npm install
npm run dev
```

或使用 Docker：

```bash
docker-compose up -d frontend
```

### 访问地址

- 🌐 **前端**: http://localhost (Docker) 或 http://localhost:5173 (开发模式)
- 📡 **后端 API**: http://localhost:8080/api
- 🗄️ **MySQL**: localhost:3306 (root/root123456)
- 💾 **Redis**: localhost:6379

### 查看日志

```bash
# 查看所有服务日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f mysql
```

### 常用命令

```bash
# 列出运行中的容器
docker-compose ps

# 进入容器
docker-compose exec backend bash
docker-compose exec mysql bash

# 重启服务
docker-compose restart

# 清理所有数据（谨慎操作）
docker-compose down -v
```

### 数据库初始化

首次启动时，Spring Boot 会自动创建表结构。

手动初始化（如需要）：

```bash
# 进入 MySQL 容器
docker-compose exec mysql mysql -uroot -proot123456

# 创建数据库
CREATE DATABASE openforum CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### 常见问题

**Q: 启动后无法访问前端**
A: 检查防火墙设置或等待容器完全启动（1-2分钟）

**Q: MySQL 连接失败**
A: 确认 MySQL 容器已启动，检查密码是否正确

**Q: 前端无法调用 API**
A: 检查后端是否启动，检查 nginx.conf 中的代理配置

**Q: 端口已被占用**
A: 修改 docker-compose.yml 中的端口映射

### 开发相关

#### 仅启动后端和数据库

```bash
docker-compose up -d mysql redis backend
cd frontend
npm run dev
```

#### 修改配置

- **后端**: 编辑 `backend/src/main/resources/application.yml`
- **前端**: 编辑 `.env.development` 或 `.env.production`

#### 打包部署

```bash
# 后端
cd backend
mvn clean package

# 前端
cd frontend
npm run build
```

### 生产部署

完整的 Docker Compose 配置已包含在项目中：

```bash
# 一键部署到生产环境
docker-compose -f docker-compose.yml up -d
```

或参考 README.md 中的部署章节进行自定义配置。

---

🎉 祝你使用愉快！如有问题欢迎提 Issue
