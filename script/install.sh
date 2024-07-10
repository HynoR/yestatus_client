#!/bin/bash

# 检查是否以 root 权限运行
if [ "$EUID" -ne 0 ]; then
  echo "请以 root 权限运行此脚本"
  exit 1
fi

# 检查 wget 是否安装
if ! command -v wget &> /dev/null; then
    echo "wget 未安装。正在安装..."
    apt-get update && apt-get install -y wget || yum install -y wget
fi

# 获取用户输入
read -p "请输入服务器地址 (默认: 127.0.0.1): " SERVER
SERVER=${SERVER:-127.0.0.1}

read -p "请输入端口 (默认: 35601): " PORT
PORT=${PORT:-35601}

read -p "请输入用户名 (默认: otto): " USER
USER=${SERVER:-otto}

read -p "请输入密码 (默认: 114514): " PASSWORD
PASSWORD=${PASSWORD:-114514}

read -p "请输入更新间隔（秒）(默认: 1): " INTERVAL
INTERVAL=${INTERVAL:-1}

# 设置安装路径
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="yestatus"
DOWNLOAD_URL="https://github.com/path/to/yestatus"  # 请替换为实际的下载 URL

# 下载程序
echo "正在下载 Server Status 程序..."
wget -O $INSTALL_DIR/$BINARY_NAME $DOWNLOAD_URL

if [ $? -ne 0 ]; then
    echo "下载失败，请检查网络连接或下载 URL 是否正确。"
    exit 1
fi

# 添加执行权限
chmod +x $INSTALL_DIR/$BINARY_NAME

# 创建 systemd service 文件
cat > /etc/systemd/system/yestatus.service <<EOL
[Unit]
Description=Server Status Reporter
After=network.target

[Service]
ExecStart=$INSTALL_DIR/$BINARY_NAME --server $SERVER --port $PORT --user $USER --password $PASSWORD --interval $INTERVAL
Restart=always
User=root

[Install]
WantedBy=multi-user.target
EOL

# 重新加载 systemd 配置
systemctl daemon-reload

# 启用并启动服务
systemctl enable yestatus.service
systemctl start yestatus.service

echo "安装完成！Server Status 服务已启动并设置为开机自启动。"
echo "你可以使用以下命令查看服务状态："
echo "systemctl status yestatus.service"