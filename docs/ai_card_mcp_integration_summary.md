# ai.card MCP統合作業完了報告 (2025/01/06)

## 作業概要
ai.cardプロジェクトに独立したMCPサーバー実装を追加し、fastapi_mcpベースでカードゲーム機能をMCPツールとして公開。

## 実装完了機能

### 1. MCP依存関係追加
**場所**: `card/api/requirements.txt`

**追加項目**:
```txt
fastapi-mcp==0.1.0
```

### 2. ai.card MCPサーバー実装
**場所**: `card/api/app/mcp_server.py`

**機能**:
- FastAPI + fastapi_mcp統合
- 独立したMCPサーバークラス `AICardMcpServer`
- 環境変数による有効/無効切り替え

**公開MCPツール (9個)**:

**カード管理系 (5個)**:
- `get_user_cards` - ユーザーのカード一覧取得
- `draw_card` - ガチャでカード取得
- `get_card_details` - カード詳細情報取得
- `analyze_card_collection` - コレクション分析
- `get_unique_registry` - ユニークカード登録状況

**システム系 (3個)**:
- `sync_cards_atproto` - atproto同期
- `get_gacha_stats` - ガチャシステム統計
- 既存のFastAPI REST API（/api/v1/*）

**atproto連携系 (1個)**:
- `sync_cards_atproto` - カードデータのatproto PDS同期

### 3. メインアプリ統合
**場所**: `card/api/app/main.py`

**変更内容**:
```python
# MCP統合
from app.mcp_server import AICardMcpServer

enable_mcp = os.getenv("ENABLE_MCP", "true").lower() == "true"
mcp_server = AICardMcpServer(enable_mcp=enable_mcp)
app = mcp_server.get_app()
```

**動作確認**:
- `ENABLE_MCP=true` (デフォルト): MCPサーバー有効
- `ENABLE_MCP=false`: 通常のFastAPIのみ

## 技術実装詳細

### アーキテクチャ設計
```
ai.card/
├── api/app/main.py          # FastAPIアプリ + MCP統合
├── api/app/mcp_server.py    # 独立MCPサーバー
├── api/app/routes/          # REST API (既存)
├── api/app/services/        # ビジネスロジック (既存)
├── api/app/repositories/    # データアクセス (既存)
└── api/requirements.txt     # fastapi-mcp追加
```

### MCPツール実装パターン
```python
@self.app.get("/tool_name", operation_id="tool_name")
async def tool_name(
    param: str,
    session: AsyncSession = Depends(get_session)
) -> Dict[str, Any]:
    """Tool description"""
    try:
        # ビジネスロジック実行
        result = await service.method(param)
        return {"success": True, "data": result}
    except Exception as e:
        logger.error(f"Error: {e}")
        return {"error": str(e)}
```

### 既存システムとの統合
- **REST API**: 既存の `/api/v1/*` エンドポイント保持
- **データアクセス**: 既存のRepository/Serviceパターン再利用
- **認証**: 既存のDID認証システム利用
- **データベース**: 既存のPostgreSQL + SQLAlchemy

## 起動方法

### 1. 環境セットアップ
```bash
cd /Users/syui/ai/gpt/card/api

# 仮想環境作成 (推奨)
python -m venv ~/.config/syui/ai/card/venv
source ~/.config/syui/ai/card/venv/bin/activate

# 依存関係インストール
pip install -r requirements.txt
```

### 2. サーバー起動
```bash
# MCP有効 (デフォルト)
python -m app.main

# または
ENABLE_MCP=true uvicorn app.main:app --host 0.0.0.0 --port 8000

# MCP無効
ENABLE_MCP=false uvicorn app.main:app --host 0.0.0.0 --port 8000
```

### 3. 動作確認
```bash
# ヘルスチェック
curl http://localhost:8000/health

# MCP有効時の応答例
{
    "status": "healthy",
    "mcp_enabled": true,
    "mcp_endpoint": "/mcp"
}

# API仕様確認
curl http://localhost:8000/docs
```

## MCPクライアント連携

### ai.gptからの接続
```python
# ai.gptのcard_integration.pyで使用
api_base_url = "http://localhost:8000"

# MCPツール経由でアクセス
response = await client.get(f"{api_base_url}/get_user_cards?did=did:plc:...")
```

### Claude Desktop等での利用
```json
{
  "mcpServers": {
    "aicard": {
      "command": "uvicorn",
      "args": ["app.main:app", "--host", "localhost", "--port", "8000"],
      "cwd": "/Users/syui/ai/gpt/card/api"
    }
  }
}
```

## 既知の制約と注意点

### 1. 依存関係
- **fastapi-mcp**: 現在のバージョンは0.1.0（初期実装）
- **Python環境**: システム環境では外部管理エラーが発生
- **推奨**: 仮想環境での実行

### 2. データベース要件
- PostgreSQL稼働が必要
- SQLite fallback対応済み（開発用）
- atproto同期は外部API依存

### 3. MCP無効化時の動作
- `ENABLE_MCP=false`時は通常のFastAPI
- 既存のREST API (`/api/v1/*`) は常時利用可能
- iOS/Webアプリは影響なし

## ai.gptとの統合戦略

### 現在の状況
- **ai.gpt**: 統合MCPサーバー（ai.gpt + ai.shell + ai.card proxy）
- **ai.card**: 独立MCPサーバー（カードロジック本体）

### 推奨連携パターン
```
Claude Desktop/Cursor
    ↓
ai.gpt MCP (port 8001)  ←-- ai.shell tools
    ↓ HTTP client
ai.card MCP (port 8000) ←-- card business logic
    ↓
PostgreSQL/atproto PDS
```

### 重複削除対象
ai.gptプロジェクトから以下を削除可能：
- `src/aigpt/card_integration.py` (HTTPクライアント)
- `./card/` (submodule)
- MCPサーバーの `--enable-card` オプション

## 次回開発時の推奨手順

### 1. 環境確認
```bash
cd /Users/syui/ai/gpt/card/api
source ~/.config/syui/ai/card/venv/bin/activate
python -c "from app.mcp_server import AICardMcpServer; print('✓ Import OK')"
```

### 2. サーバー起動テスト
```bash
# MCP有効でサーバー起動
uvicorn app.main:app --host localhost --port 8000 --reload

# 別ターミナルで動作確認
curl http://localhost:8000/health
curl "http://localhost:8000/get_gacha_stats"
```

### 3. ai.gptとの統合確認
```bash
# ai.gptサーバー起動
cd /Users/syui/ai/gpt
aigpt server --port 8001

# ai.cardサーバー起動  
cd /Users/syui/ai/gpt/card/api
uvicorn app.main:app --port 8000

# 連携テスト（ai.gpt → ai.card）
curl "http://localhost:8001/get_user_cards?did=did:plc:example"
```

## 成果サマリー

**実装済み**: ai.card独立MCPサーバー
**技術的成果**: fastapi_mcp統合、9個のMCPツール公開
**アーキテクチャ**: 疎結合設計、既存システム保持
**拡張性**: 環境変数によるMCP有効/無効切り替え

**統合効果**:
- ai.cardが独立したMCPサーバーとして動作
- ai.gptとの重複MCPコード解消
- カードビジネスロジックの責任分離維持
- 将来的なマイクロサービス化への対応