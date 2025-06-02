# ai.card と ai.gpt の統合ガイド

## 概要

ai.gptのMCPサーバーにai.cardのツールを統合し、AIがカードゲームシステムとやり取りできるようになりました。

## セットアップ

### 1. 必要な環境

- Python 3.13
- ai.gpt プロジェクト
- ai.card プロジェクト（`./card` ディレクトリ）

### 2. 起動手順

**ステップ1: ai.cardサーバーを起動**（ターミナル1）
```bash
cd card
./start_server.sh
```

**ステップ2: ai.gpt MCPサーバーを起動**（ターミナル2）
```bash
aigpt server
```

起動時に以下が表示されることを確認：
- 🎴 Card Game System: 6 tools
- 🎴 ai.card: ./card directory detected

**ステップ3: AIと対話**（ターミナル3）
```bash
aigpt conv syui --provider openai
```

## 使用可能なコマンド

### カード関連の質問例

```
# カードコレクションを表示
「カードコレクションを見せて」
「私のカードを見せて」
「カード一覧を表示して」

# ガチャを実行
「ガチャを引いて」
「カードを引きたい」

# コレクション分析
「私のコレクションを分析して」

# ガチャ統計
「ガチャの統計を見せて」
```

## 技術仕様

### MCP ツール一覧

| ツール名 | 説明 | パラメータ |
|---------|------|-----------|
| `card_get_user_cards` | ユーザーのカード一覧取得 | did, limit |
| `card_draw_card` | ガチャでカード取得 | did, is_paid |
| `card_get_card_details` | カード詳細情報取得 | card_id |
| `card_analyze_collection` | コレクション分析 | did |
| `card_get_gacha_stats` | ガチャ統計取得 | なし |
| `card_system_status` | システム状態確認 | なし |

### 動作の流れ

1. **ユーザーがカード関連の質問をする**
   - AIがキーワード（カード、コレクション、ガチャなど）を検出

2. **AIが適切なMCPツールを呼び出す**
   - OpenAIのFunction Callingを使用
   - didパラメータには会話相手のユーザーID（例：'syui'）を使用

3. **ai.gpt MCPサーバーがai.cardサーバーに転送**
   - http://localhost:8001 → http://localhost:8000
   - 適切なエンドポイントにリクエストを転送

4. **結果をAIが解釈して返答**
   - カード情報を分かりやすく説明
   - エラー時は適切なガイダンスを提供

## 設定

### config.json

```json
{
  "providers": {
    "openai": {
      "api_key": "your-api-key",
      "default_model": "gpt-4o-mini",
      "system_prompt": "カード関連の質問では、必ずcard_get_user_cardsなどのツールを使用してください。"
    }
  },
  "mcp": {
    "servers": {
      "ai_gpt": {
        "endpoints": {
          "card_get_user_cards": "/card_get_user_cards",
          "card_draw_card": "/card_draw_card",
          "card_get_card_details": "/card_get_card_details",
          "card_analyze_collection": "/card_analyze_collection",
          "card_get_gacha_stats": "/card_get_gacha_stats",
          "card_system_status": "/card_system_status"
        }
      }
    }
  }
}
```

## トラブルシューティング

### エラー: "ai.card server is not running"

ai.cardサーバーが起動していません。以下を実行：
```bash
cd card
./start_server.sh
```

### エラー: "カード一覧の取得に失敗しました"

1. ai.cardサーバーが正常に起動しているか確認
2. aigpt serverを再起動
3. ポート8000と8001が使用可能か確認

### プロセスの終了方法

```bash
# ポート8001のプロセスを終了
lsof -ti:8001 | xargs kill -9

# ポート8000のプロセスを終了
lsof -ti:8000 | xargs kill -9
```

## 実装の詳細

### 主な変更点

1. **ai.gpt MCPサーバーの拡張** (`src/aigpt/mcp_server.py`)
   - `./card`ディレクトリの存在を検出
   - ai.card用のMCPツールを自動登録

2. **AIプロバイダーの更新** (`src/aigpt/ai_provider.py`)
   - card_*ツールの定義追加
   - ツール実行時のパラメータ処理

3. **MCPクライアントの拡張** (`src/aigpt/cli.py`)
   - `has_card_tools`プロパティ追加
   - ai.card MCPメソッドの実装

## 今後の拡張案

- [ ] カードバトル機能の追加
- [ ] カードトレード機能
- [ ] レアリティ別の表示
- [ ] カード画像の表示対応
- [ ] atproto連携の実装

## 関連ドキュメント

- [ai.card 開発ガイド](./card/claude.md)
- [エコシステム統合設計書](./CLAUDE.md)
- [ai.gpt README](./README.md)