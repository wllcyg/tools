#!/bin/bash

# 串口调试助手发布脚本
# 使用方法: ./release.sh 1.0.0

set -e

if [ -z "$1" ]; then
  echo "❌ 请提供版本号"
  echo "使用方法: ./release.sh 1.0.0"
  exit 1
fi

VERSION=$1
TAG="v${VERSION}"

echo "🚀 准备发布版本: ${TAG}"
echo ""

# 检查是否有未提交的改动
if [[ -n $(git status -s) ]]; then
  echo "⚠️  检测到未提交的改动:"
  git status -s
  echo ""
  read -p "是否继续? (y/n) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 取消发布"
    exit 1
  fi
  
  echo "📝 提交所有改动..."
  git add .
  git commit -m "chore: 准备发布 ${TAG}"
fi

# 创建标签
echo "🏷️  创建 Git 标签..."
git tag -a "${TAG}" -m "Release ${TAG}"

# 推送到远程
echo "⬆️  推送到 GitHub..."
git push origin main
git push origin "${TAG}"

echo ""
echo "✅ 发布成功！"
echo ""
echo "📦 GitHub Actions 正在自动构建 Windows 安装包..."
echo "🔗 查看构建进度: https://github.com/wllcyg/tools/actions"
echo "🔗 构建完成后可在 Release 页面下载: https://github.com/wllcyg/tools/releases/tag/${TAG}"
echo ""
echo "提示："
echo "  1. 等待 GitHub Actions 构建完成（约 5-10 分钟）"
echo "  2. 访问 Release 页面查看和下载安装包"
echo "  3. 提供给用户的下载链接:"
echo "     https://github.com/wllcyg/tools/releases/latest"
