Memory-Enhanced MCP Server 使用ガイド
概要
このMCPサーバーは、ChatGPTの会話履歴を記憶として保存し、AIとの対話で活用できる機能を提供します。

セットアップ
1. 依存関係のインストール
bash
pip install -r requirements.txt
2. サーバーの起動
bash
python mcp/server.py
サーバーは http://localhost:5000 で起動します。

使用方法
1. ChatGPTの会話履歴をインポート
ChatGPTから会話をエクスポートし、JSONファイルとして保存してください。

bash
# 単一ファイルをインポート
python mcp/memory_client.py import your_chatgpt_export.json

# インポート結果の例
✅ Imported 5/5 conversations
2. 記憶の検索
bash
# キーワードで記憶を検索
python mcp/memory_client.py search "プログラミング"

# 検索結果の例
🔍 Searching for: プログラミング
📚 Found 3 memories:
  • Pythonの基礎学習
    Summary: Conversation with 10 user messages and 8 assistant responses...
    Messages: 18
3. 記憶一覧の表示
bash
python mcp/memory_client.py list

# 結果の例
📋 Listing all memories...
📚 Total memories: 15
  • day
    Source: chatgpt
    Messages: 2
    Imported: 2025-01-21T10:30:45.123456
4. 記憶の詳細表示
bash
python mcp/memory_client.py detail "/path/to/memory/file.json"

# 結果の例
📄 Getting details for: /path/to/memory/file.json
Title: day
Source: chatgpt
Summary: Conversation with 1 user messages and 1 assistant responses...
Messages: 2

Recent messages:
  user: こんにちは...
  assistant: こんにちは〜！✨...
5. 記憶を活用したチャット
bash
python mcp/memory_client.py chat "Pythonについて教えて"

# 結果の例
💬 Chatting with memory: Pythonについて教えて
🤖 Response: Enhanced response with memory context...
📚 Memories used: 2
API エンドポイント
POST /memory/import/chatgpt
ChatGPTの会話履歴をインポート

json
{
  "conversation_data": { ... }
}
POST /memory/search
記憶を検索

json
{
  "query": "検索キーワード",
  "limit": 10
}
GET /memory/list
すべての記憶をリスト

GET /memory/detail?filepath=/path/to/file
記憶の詳細を取得

POST /chat
記憶を活用したチャット

json
{
  "message": "メッセージ",
  "model": "model_name"
}
記憶の保存場所
記憶は以下のディレクトリに保存されます：

~/.config/aigpt/memory/chatgpt/
各会話は個別のJSONファイルとして保存され、以下の情報を含みます：

タイトル
インポート時刻
メッセージ履歴
自動生成された要約
メタデータ
ChatGPTの会話エクスポート方法
ChatGPTの設定画面を開く
"Data controls" → "Export data" を選択
エクスポートファイルをダウンロード
conversations.json ファイルを使用
拡張可能な機能
高度な検索: ベクトル検索やセマンティック検索の実装
要約生成: AIによる自動要約の改善
記憶の分類: カテゴリやタグによる分類
記憶の統合: 複数の会話からの知識統合
プライバシー保護: 機密情報の自動検出・マスキング
トラブルシューティング
サーバーが起動しない
ポート5000が使用中でないか確認
依存関係が正しくインストールされているか確認
インポートに失敗する
JSONファイルが正しい形式か確認
ファイルパスが正しいか確認
ファイルの権限を確認
検索結果が表示されない
インポートが正常に完了しているか確認
検索キーワードを変更して試行
