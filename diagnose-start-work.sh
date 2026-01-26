#!/bin/bash

echo "=== start-work 命令诊断工具 ==="
echo ""

echo "1. 检查配置文件"
echo "----------------"
echo "项目级插件配置:"
cat .opencode/oh-my-opencode.json 2>/dev/null || echo "❌ 文件不存在"
echo ""

echo "用户级 OpenCode 配置:"
cat C:/Users/mumu/.config/opencode/opencode.json 2>/dev/null || echo "❌ 文件不存在"
echo ""

echo "Boulder 状态文件:"
cat .sisyphus/boulder.json 2>/dev/null || echo "❌ 文件不存在"
echo ""

echo "2. 检查计划文件"
echo "----------------"
ls -la .sisyphus/plans/ 2>/dev/null || echo "❌ 目录不存在"
echo ""

echo "3. 检查插件目录"
echo "----------------"
echo "oh-my-opencode 插件:"
ls -la C:/Users/mumu/.config/opencode/node_modules/oh-my-opencode/dist/hooks/start-work/ 2>/dev/null || echo "❌ 目录不存在"
echo ""

echo "4. 检查命令目录"
echo "----------------"
echo "用户命令目录:"
ls -la C:/Users/mumu/.claude/commands/ 2>/dev/null || echo "❌ 目录不存在"
echo ""

echo "项目命令目录:"
ls -la .opencode/command/ 2>/dev/null || echo "❌ 目录不存在"
echo ""

echo "5. 检查 OpenCode 版本"
echo "----------------"
which opencode && opencode --version 2>/dev/null || echo "❌ opencode 命令未找到"
echo ""

echo "6. 检查 Rust 项目状态"
echo "----------------"
cargo check 2>&1 | tail -5
echo ""

echo "=== 诊断完成 ==="
echo ""
echo "建议操作:"
echo "1. 重启 OpenCode 会话"
echo "2. 在 OpenCode 中输入 /start-work"
echo "3. 如果仍然失败，尝试 /plan 'test' 测试 Prometheus"
echo "4. 检查 OpenCode 控制台输出错误信息"
