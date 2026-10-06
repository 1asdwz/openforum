#!/bin/bash

echo "🛑 OpenForum 停止脚本"
echo "================================"

docker-compose down -v

echo "✅ 所有容器已停止"
echo "💡 使用 'bash start.sh' 重新启动"
