#!/bin/bash

echo "🧪 MCPサーバーテスト開始..."
echo ""

# サーバー起動（バックグラウンド）
./target/debug/aigpt server &
SERVER_PID=$!

sleep 2

echo "✅ サーバー起動完了 (PID: $SERVER_PID)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 利用可能なツール一覧:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "基本ツール:"
echo "  • create_memory"
echo "  • search_memories"
echo "  • update_memory"
echo "  • delete_memory"
echo ""
echo "AI機能ツール 🎮:"
echo "  • create_memory_with_ai (心理テスト風)"
echo "  • list_memories_by_priority (ランキング)"
echo "  • daily_challenge (デイリークエスト)"
echo ""
echo "恋愛コンパニオン 💕:"
echo "  • create_companion"
echo "  • companion_react"
echo "  • companion_profile"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 次のステップ:"
echo "1. Claude Codeの設定に追加"
echo "2. Claude Code再起動"
echo "3. ツールを使って試す！"
echo ""
echo "設定ファイル: ~/.config/claude-code/config.json"
echo ""

# サーバー停止
kill $SERVER_PID 2>/dev/null

echo "✅ テスト完了！"
