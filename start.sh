#!/bin/bash

echo "🚀 OpenForum 快速启动脚本"
echo "================================"

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装，请先安装 Docker"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose 未安装，请先安装 Docker Compose"
    exit 1
fi

echo "✅ Docker 环境检查完毕"

# 停止现有容器
echo "🛑 停止现有容器..."
docker-compose down

# 构建镜像
echo "🔨 构建 Docker 镜像..."
docker-compose build --no-cache

# 启动容器
echo "🚀 启动容器..."
docker-compose up -d

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 10

# 检查服务状态
echo ""
echo "📊 服务状态检查："
echo "================================"

# 检查 MySQL
if docker-compose exec -T mysql mysqladmin ping -h localhost &> /dev/null; then
    echo "✅ MySQL: 运行中"
else
    echo "❌ MySQL: 启动失败"
fi

# 检查 Redis
if docker-compose exec -T redis redis-cli ping &> /dev/null; then
    echo "✅ Redis: 运行中"
else
    echo "❌ Redis: 启动失败"
fi

# 检查后端
if curl -s http://localhost:8080/api/health &> /dev/null; then
    echo "✅ Backend: 运行中 (http://localhost:8080)"
else
    echo "⏳ Backend: 启动中..."
fi

# 检查前端
if curl -s http://localhost/health &> /dev/null; then
    echo "✅ Frontend: 运行中 (http://localhost)"
else
    echo "⏳ Frontend: 启动中..."
fi

echo ""
echo "================================"
echo "✅ OpenForum 启动完成！"
echo ""
echo "访问地址："
echo "  🌐 前端: http://localhost"
echo "  📡 后端 API: http://localhost:8080/api"
echo "  🗄️  MySQL: localhost:3306"
echo "  💾 Redis: localhost:6379"
echo ""
echo "默认账户："
echo "  数据库: root / root123456"
echo ""
echo "查看日志:"
echo "  docker-compose logs -f"
echo ""
echo "停止服务:"
echo "  docker-compose down"
echo ""
