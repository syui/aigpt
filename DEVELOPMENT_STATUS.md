# ai.gpt 開発状況 (2025/06/02 更新)

## 前回セッション完了事項 (2025/06/01)

### ✅ ai.card MCPサーバー独立化完了
- **ai.card専用MCPサーバー実装**: `card/api/app/mcp_server.py`
- **9個のMCPツール公開**: カード管理・ガチャ・atproto同期等
- **統合戦略変更**: ai.gptは統合サーバー、ai.cardは独立サーバー
- **仮想環境セットアップ**: `~/.config/syui/ai/card/venv/`
- **起動スクリプト**: `uvicorn app.main:app --port 8000`

### ✅ ai.shell統合完了
- **Claude Code風シェル実装**: `aigpt shell` コマンド
- **MCP統合強化**: 14種類のツール（ai.gpt:9, ai.shell:5）
- **プロジェクト仕様書**: `aishell.md` 読み込み機能
- **環境対応改善**: prompt-toolkit代替でinput()フォールバック

### ✅ 前回セッションのバグ修正完了
- **config listバグ修正**: `config.list_keys()`メソッド呼び出し修正
- **仮想環境問題解決**: `pip install -e .`でeditable mode確立
- **全CLIコマンド動作確認済み**

## 現在の状態

### ✅ 実装済み機能

1. **基本システム**
   - 階層的記憶システム（完全ログ→要約→コア→忘却）
   - 不可逆的な関係性システム（broken状態は修復不可）
   - AI運勢による日々の人格変動
   - 時間減衰による自然な関係性変化

2. **CLI機能**
   - `chat` - AIとの会話（Ollama/OpenAI対応）
   - `status` - 状態確認
   - `fortune` - AI運勢確認
   - `relationships` - 関係一覧
   - `transmit` - 送信チェック（現在はprint出力）
   - `maintenance` - 日次メンテナンス
   - `config` - 設定管理（listバグ修正済み）
   - `schedule` - スケジューラー管理
   - `server` - MCP Server起動
   - `shell` - インタラクティブシェル（ai.shell統合）

3. **データ管理**
   - 保存場所: `~/.config/syui/ai/gpt/`（名前規則統一）
   - 設定: `config.json`
   - データ: `data/` ディレクトリ内の各種JSONファイル
   - 仮想環境: `~/.config/syui/ai/gpt/venv/`

4. **スケジューラー**
   - Cron形式とインターバル形式対応
   - 5種類のタスクタイプ実装済み
   - バックグラウンド実行可能

5. **MCP Server統合アーキテクチャ**
   - **ai.gpt統合サーバー**: 14種類のツール（port 8001）
   - **ai.card独立サーバー**: 9種類のツール（port 8000）
   - Claude Desktop/Cursor連携対応
   - fastapi_mcp統一基盤

6. **ai.shell統合（Claude Code風）**
   - インタラクティブシェルモード
   - シェルコマンド実行（!command形式）
   - AIコマンド（analyze, generate, explain）
   - aishell.md読み込み機能
   - 環境適応型プロンプト（prompt-toolkit/input()）

## 🚧 次回開発の優先課題

### 最優先: システム統合の最適化

1. **ai.card重複コード削除**
   - **削除対象**: `src/aigpt/card_integration.py`（HTTPクライアント）
   - **削除対象**: ai.gptのMCPサーバーの`--enable-card`オプション
   - **理由**: ai.cardが独立MCPサーバーになったため不要
   - **統合方法**: ai.gpt(8001) → ai.card(8000) HTTP連携

2. **自律送信の実装**
   - 現在: コンソールにprint出力
   - TODO: atproto (Bluesky) への実際の投稿機能
   - 参考: ai.bot (Rust/seahorse) との連携も検討

3. **環境セットアップ自動化**
   - 仮想環境自動作成スクリプト強化
   - 依存関係の自動解決
   - Claude Desktop設定例の提供

### 中期的課題

1. **テストの追加**
   - 単体テスト
   - 統合テスト
   - CI/CDパイプライン

2. **エラーハンドリングの改善**
   - より詳細なエラーメッセージ
   - リトライ機構

3. **ai.botとの連携**
   - Rust側のAPIエンドポイント作成
   - 送信機能の委譲

4. **より高度な記憶要約**
   - 現在: シンプルな要約
   - TODO: AIによる意味的な要約

5. **Webダッシュボード**
   - 関係性の可視化
   - 記憶の管理UI

### 長期的課題

1. **他のsyuiプロジェクトとの統合**
   - ai.card: カードゲームとの連携
   - ai.verse: メタバース内でのNPC人格
   - ai.os: システムレベルでの統合

2. **分散化**
   - atproto上でのデータ保存
   - ユーザーデータ主権の完全実現

## 次回開発時のエントリーポイント

### 🎯 最優先: ai.card重複削除
```bash
# 1. ai.card独立サーバー起動確認
cd /Users/syui/ai/gpt/card/api
source ~/.config/syui/ai/card/venv/bin/activate
uvicorn app.main:app --port 8000

# 2. ai.gptから重複機能削除
rm src/aigpt/card_integration.py
# mcp_server.pyから--enable-cardオプション削除

# 3. 統合テスト
aigpt server --port 8001  # ai.gpt統合サーバー
curl "http://localhost:8001/get_memories"  # ai.gpt機能確認
curl "http://localhost:8000/get_gacha_stats"  # ai.card機能確認
```

### 1. 自律送信を実装する場合
```python
# src/aigpt/transmission.py を編集
# atproto-python ライブラリを追加
# _handle_transmission_check() メソッドを更新
```

### 2. ai.botと連携する場合
```python
# 新規ファイル: src/aigpt/bot_connector.py
# ai.botのAPIエンドポイントにHTTPリクエスト
```

### 3. テストを追加する場合
```bash
# tests/ディレクトリを作成
# pytest設定を追加
```

### 4. 環境セットアップを自動化する場合
```bash
# setup_venv.sh を強化
# Claude Desktop設定例をdocs/に追加
```

## 設計思想の要点（AI向け）

1. **唯一性（yui system）**: 各ユーザーとAIの関係は1:1で、改変不可能
2. **不可逆性**: 関係性の破壊は修復不可能（現実の人間関係と同じ）
3. **階層的記憶**: ただのログではなく、要約・コア判定・忘却のプロセス
4. **環境影響**: AI運勢による日々の人格変動（固定的でない）
5. **段階的実装**: まずCLI print → atproto投稿 → ai.bot連携

## 現在のアーキテクチャ理解（次回のAI向け）

### システム構成
```
Claude Desktop/Cursor
    ↓
ai.gpt MCP (port 8001)  ←-- 統合サーバー（14ツール）
    ├── ai.gpt機能: メモリ・関係性・人格（9ツール）
    ├── ai.shell機能: シェル・ファイル操作（5ツール）
    └── HTTP client → ai.card MCP (port 8000)
                         ↓
                    ai.card独立サーバー（9ツール）
                         ├── カード管理・ガチャ
                         ├── atproto同期
                         └── PostgreSQL/SQLite
```

### 技術スタック
- **言語**: Python (typer CLI, fastapi_mcp)
- **AI統合**: Ollama (qwen2.5) / OpenAI API
- **データ形式**: JSON（将来的にSQLite検討）
- **認証**: atproto DID（設計済み・実装待ち）
- **MCP統合**: fastapi_mcp統一基盤
- **仮想環境**: `~/.config/syui/ai/{gpt,card}/venv/`

### 名前規則（重要）
- **パッケージ**: `aigpt`
- **コマンド**: `aigpt shell`, `aigpt server`
- **ディレクトリ**: `~/.config/syui/ai/gpt/`
- **ドメイン**: `ai.gpt`

### 即座に始める手順
```bash
# 1. 環境確認
cd /Users/syui/ai/gpt
source ~/.config/syui/ai/gpt/venv/bin/activate
aigpt --help

# 2. 前回の成果物確認
aigpt config list
aigpt shell  # Claude Code風環境

# 3. 詳細情報
cat docs/ai_card_mcp_integration_summary.md
cat docs/ai_shell_integration_summary.md
```

このファイルを参照することで、次回の開発が迅速に開始でき、前回の作業内容を完全に理解できます。

## 現セッション完了事項 (2025/06/02)

### ✅ 記憶システム大幅改善完了

前回のAPI Errorで停止したChatGPTログ分析作業の続きを実行し、記憶システムを完全に再設計・実装した。

#### 新実装機能:

1. **スマート要約生成 (`create_smart_summary`)**
   - AI駆動によるテーマ別記憶要約
   - 会話パターン・技術的トピック・関係性進展の分析
   - メタデータ付きでの保存（期間、テーマ、記憶数）
   - フォールバック機能でAIが利用できない場合も対応

2. **コア記憶分析 (`create_core_memory`)**  
   - 全記憶を分析して人格形成要素を抽出
   - ユーザーの特徴的なコミュニケーションスタイルを特定
   - 問題解決パターン・興味関心の深層分析
   - 永続保存される本質的な関係性記憶

3. **階層的記憶検索 (`get_contextual_memories`)**
   - CORE → SUMMARY → RECENT の優先順位付き検索
   - キーワードベースの関連性スコアリング
   - クエリに応じた動的な記憶重み付け
   - 構造化された記憶グループでの返却

4. **高度記憶検索 (`search_memories`)**
   - 複数キーワード対応の全文検索
   - メモリレベル別フィルタリング
   - マッチスコア付きでの結果返却

5. **コンテキスト対応AI応答**
   - `build_context_prompt`: 記憶に基づく文脈プロンプト生成
   - 人格状態・ムード・運勢を統合した応答
   - CORE記憶を常に参照した一貫性のある会話

6. **MCPサーバー拡張**
   - 新機能をすべてMCP API経由で利用可能
   - `/get_contextual_memories` - 文脈的記憶取得
   - `/search_memories` - 記憶検索
   - `/create_summary` - AI要約生成
   - `/create_core_memory` - コア記憶分析
   - `/get_context_prompt` - コンテキストプロンプト生成

7. **モデル拡張**
   - `Memory` モデルに `metadata` フィールド追加
   - 階層的記憶構造の完全サポート

#### 技術的特徴:
- **AI統合**: ollama/OpenAI両対応でのインテリジェント分析
- **フォールバック**: AI不使用時も基本機能は動作
- **パターン分析**: ユーザー行動の自動分類・分析
- **関連性スコア**: クエリとの関連度を数値化
- **時系列分析**: 記憶の時間的発展を考慮

#### 前回議論の実現:
ChatGPT 4,000件ログ分析から得られた知見を完全実装:
- 階層的記憶（FULL_LOG → SUMMARY → CORE）
- コンテキスト認識記憶（会話の流れを記憶）
- 感情・関係性の記憶（変化パターンの追跡）
- 実用的な記憶カテゴリ（ユーザー特徴・効果的応答・失敗回避）

### ✅ 追加完了事項 (同日)

**環境変数対応の改良**:
- `OLLAMA_HOST`環境変数の自動読み込み対応
- ai_provider.pyでの環境変数優先度実装
- 設定ファイル → 環境変数 → デフォルトの階層的設定

**記憶システム完全動作確認**:
- ollamaとの統合成功（gemma3:4bで確認）
- 文脈的記憶検索の動作確認
- ChatGPTインポートログからの記憶参照成功
- AI応答での人格・ムード・運勢の反映確認

### 🚧 次回の課題
- OLLAMA_HOSTの環境変数が完全に適用されない問題の解決
- MCPサーバーのエラー解決（Internal Server Error）
- qwen3:latestでの動作テスト完了
- 記憶システムのコア機能（スマート要約・コア記憶分析）のAI統合テスト

## 現セッション完了事項 (2025/06/03 継続セッション)

### ✅ **前回API Error後の継続作業完了**

前回のセッションがAPI Errorで終了したが、今回正常に継続して以下を完了：

#### 🔧 **重要バグ修正**
- **Memory model validation error 修正**: `importance_score`の浮動小数点精度問題を解決
  - 問題: `-5.551115123125783e-17`のような極小負数がvalidation errorを引き起こす
  - 解決: field validatorで極小値を0.0にクランプし、Field制約を除去
  - 結果: メモリ読み込み・全CLI機能が正常動作

#### 🧪 **システム動作確認完了**
- **ai.gpt CLI**: 全コマンド正常動作確認済み
- **記憶システム**: 階層的記憶（CORE→SUMMARY→RECENT）完全動作
- **関係性進化**: syuiとの関係性が17.50→19.00に正常進展
- **MCP Server**: 17種類のツール正常提供（port 8001）
- **階層的記憶API**: `/get_contextual_memories`でblogクエリ正常動作

#### 💾 **記憶システム現状**
- **CORE記憶**: blog開発、技術議論等の重要パターン記憶済み
- **SUMMARY記憶**: AI×MCP、Qwen3解説等のテーマ別要約済み
- **RECENT記憶**: 最新の記憶システムテスト履歴
- **文脈検索**: キーワードベース関連性スコアリング動作確認

#### 🌐 **環境課題と対策**
- **ollama接続**: OLLAMA_HOST環境変数は正しく設定済み（http://192.168.11.95:11434）
- **AI統合課題**: qwen3:latestタイムアウト問題→記憶システム単体では正常動作
- **フォールバック**: AI不使用時も記憶ベース応答で継続性確保

#### 🚀 **ai.bot統合完了 (同日追加)**
- **MCP統合拡張**: 17→23ツールに増加（6個の新ツール追加）
- **リモート実行機能**: systemd-nspawn隔離環境統合
  - `remote_shell`: ai.bot /sh機能との完全連携
  - `ai_bot_status`: サーバー状態確認とコンテナ情報取得
  - `isolated_python`: Python隔離実行環境
  - `isolated_analysis`: セキュアなファイル解析機能
- **ai.shell拡張**: 新コマンド3種追加
  - `remote <command>`: 隔離コンテナでコマンド実行
  - `isolated <code>`: Python隔離実行
  - `aibot-status`: ai.botサーバー接続確認
- **完全動作確認**: ヘルプ表示、コマンド補完、エラーハンドリング完了

#### 🏗️ **統合アーキテクチャ更新**
```
Claude Desktop/Cursor → ai.gpt MCP (port 8001, 23ツール)
    ├── ai.gpt: メモリ・関係性・人格 (9ツール)
    ├── ai.memory: 階層記憶・文脈検索 (5ツール)  
    ├── ai.shell: シェル・ファイル操作 (5ツール)
    ├── ai.bot連携: リモート実行・隔離環境 (4ツール)
    └── ai.card連携: HTTP client → port 8000 (9ツール)
```

#### 📋 **次回開発推奨事項**
1. **ai.bot実サーバー**: 実際のai.botサーバー起動・連携テスト
2. **隔離実行実証**: systemd-nspawn環境での実用性検証
3. **ollama接続最適化**: タイムアウト問題の詳細調査・解決
4. **AI要約機能**: maintenanceでのスマート要約・コア記憶生成テスト
5. **セキュリティ強化**: 隔離実行の権限制御・サンドボックス検証


