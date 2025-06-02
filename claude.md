# エコシステム統合設計書

## 中核思想
- **存在子理論**: この世界で最も小さいもの（存在子/ai）の探求
- **唯一性原則**: 現実の個人の唯一性をすべてのシステムで担保
- **現実の反映**: 現実→ゲーム→現実の循環的影響

## システム構成図

```
存在子(ai) - 最小単位の意識
    ↓
[ai.moji] 文字システム
    ↓
[ai.os] + [ai.game device] ← 統合ハードウェア
    ├── ai.shell (Claude Code的機能)
    ├── ai.gpt (自律人格・記憶システム)
    ├── ai.ai (個人特化AI・心を読み取るAI)
    ├── ai.card (カードゲーム・iOS/Web/API)
    └── ai.bot (分散SNS連携・カード配布)
    ↓
[ai.verse] メタバース
    ├── world system (惑星型3D世界)
    ├── at system (atproto/分散SNS)
    ├── yui system (唯一性担保)
    └── ai system (存在属性)
```

## 名前規則

名前規則は他のprojectと全て共通しています。exampleを示しますので、このルールに従ってください。

ここでは`ai.os`の場合の名前規則の例を記述します。

name: ai.os

**[ "package", "code", "command" ]**: aios
**[ "dir", "url" ]**: ai/os
**[ "domain", "json" ]**: ai.os

```sh
$ curl -sL https://git.syui.ai/ai/ai/raw/branch/main/ai.json|jq .ai.os
{ "type": "os" }
```

```json
{
  "ai": {
    "os":{}
  }
}
```

他のprojectも同じ名前規則を採用します。`ai.gpt`ならpackageは`aigpt`です。

## config(設定ファイル, env, 環境依存)

`config`を置く場所は統一されており、各projectの名前規則の`dir`項目を使用します。例えば、aiosの場合は`~/.config/syui/ai/os/`以下となります。pythonなどを使用する場合、`python -m venv`などでこのpackage config dirに環境を構築して実行するようにしてください。

domain形式を採用して、私は各projectを`git.syui.ai/ai`にhostしていますから、`~/.config/syui/ai`とします。

```sh
[syui.ai]
syui/ai
```

```sh
# example
~/.config/syui/ai
    ├── card
    ├── gpt
    ├── os
    └── shell
```

## 各システム詳細

### ai.gpt - 自律的送信AI
**目的**: 関係性に基づく自発的コミュニケーション

**中核概念**:
- **人格**: 記憶（過去の発話）と関係性パラメータで構成
- **唯一性**: atproto accountとの1:1紐付け、改変不可能
- **自律送信**: 関係性が閾値を超えると送信機能が解禁

**技術構成**:
- `MemoryManager`: 完全ログ→AI要約→コア判定→選択的忘却
- `RelationshipTracker`: 時間減衰・日次制限付き関係性スコア
- `TransmissionController`: 閾値判定・送信トリガー
- `Persona`: AI運勢（1-10ランダム）による人格変動

**実装仕様**:
```
- 言語: Python (fastapi_mcp)
- ストレージ: JSON/SQLite選択式
- インターフェース: Python CLI (click/typer)
- スケジューリング: cron-like自律処理
```

### ai.card - カードゲームシステム
**目的**: atproto基盤でのユーザーデータ主権カードゲーム

**現在の状況**:
- ai.botの機能として実装済み
- atproto accountでmentionすると1日1回カードを取得
- ai.api (MCP server予定) でユーザー管理

**移行計画**:
- **iOS移植**: Claudeが担当予定
- **データ保存**: atproto collection recordに保存（ユーザーがデータを所有）
- **不正防止**: OAuth 2.1 scope (実装待ち) + MCP serverで対応
- **画像ファイル**: Cloudflare Pagesが最適

**yui system適用**:
- カードの効果がアカウント固有
- 改ざん防止によるゲームバランス維持
- 将来的にai.verseとの統合で固有スキルと連動

### ai.ai - 心を読み取るAI
**目的**: 個人特化型AI・深層理解システム

**ai.gptとの関係**:
- ai.gpt → ai.ai: 自律送信AIから心理分析AIへの連携
- 関係性パラメータの深層分析
- ユーザーの思想コア部分の特定支援

### ai.verse - UEメタバース
**目的**: 現実反映型3D世界

**yui system実装**:
- キャラクター ↔ プレイヤー 1:1紐付け
- unique skill: そのプレイヤーのみ使用可能
- 他プレイヤーは同キャラでも同スキル使用不可

**統合要素**:
- ai.card: ゲーム内アイテムとしてのカード
- ai.gpt: NPCとしての自律AI人格
- atproto: ゲーム内プロフィール連携

## データフロー設計

### 唯一性担保の実装
```
現実の個人 → atproto account (DID) → ゲーム内avatar → 固有スキル
    ↑_______________________________|  (現実の反映)
```

### AI駆動変換システム
```
遊び・創作活動 → ai.gpt分析 → 業務成果変換 → 企業価値創出
    ↑________________________|  (Play-to-Work)
```

### カードゲーム・データ主権フロー
```
ユーザー → ai.bot mention → カード生成 → atproto collection → ユーザー所有
    ↑                                ↓
    ← iOS app表示 ← ai.card API ←
```

## 技術スタック統合

### Core Infrastructure
- **OS**: Rust-based ai.os (Arch Linux base)
- **Container**: Docker image distribution
- **Identity**: atproto selfhost server + DID管理
- **AI**: fastapi_mcp server architecture
- **CLI**: Python unified (click/typer) - Rustから移行

### Game Engine Integration
- **Engine**: Unreal Engine (Blueprint)
- **Data**: atproto → UE → atproto sync
- **Avatar**: 分散SNS profile → 3D character
- **Streaming**: game screen = broadcast screen

### Mobile/Device
- **iOS**: ai.card移植 (Claude担当)
- **Hardware**: ai.game device (future)
- **Interface**: controller-first design

## 実装優先順位

### Phase 1: AI基盤強化 (現在進行)
- [ ] ai.gpt memory system完全実装
  - 記憶の階層化（完全ログ→要約→コア→忘却）
  - 関係性パラメータの時間減衰システム
  - AI運勢による人格変動機能
- [ ] ai.card iOS移植
  - atproto collection record連携
  - MCP server化（ai.api刷新）
- [ ] fastapi_mcp統一基盤構築

### Phase 2: ゲーム統合
- [ ] ai.verse yui system実装
  - unique skill機能
  - atproto連携強化
- [ ] ai.gpt ↔ ai.ai連携機能
- [ ] 分散SNS ↔ ゲーム同期

### Phase 3: メタバース浸透
- [ ] VTuber配信機能統合
- [ ] Play-to-Work変換システム
- [ ] ai.game device prototype

## 将来的な連携構想

### システム間連携（現在は独立実装）
```
ai.gpt (自律送信) ←→ ai.ai (心理分析)
ai.card (iOS,Web,API) ←→ ai.verse (UEゲーム世界)
```

**共通基盤**: fastapi_mcp
**共通思想**: yui system（現実の反映・唯一性担保）

### データ改ざん防止戦略
- **短期**: MCP serverによる検証
- **中期**: OAuth 2.1 scope実装待ち
- **長期**: ブロックチェーン的整合性チェック

## AIコミュニケーション最適化

### プロジェクト要件定義テンプレート
```markdown
# [プロジェクト名] 要件定義

## 哲学的背景
- 存在子理論との関連：
- yui system適用範囲：
- 現実反映の仕組み：

## 技術要件
- 使用技術（fastapi_mcp統一）：
- atproto連携方法：
- データ永続化方法：

## ユーザーストーリー
1. ユーザーが...すると
2. システムが...を実行し
3. 結果として...が実現される

## 成功指標
- 技術的：
- 哲学的（唯一性担保）：
```

### Claude Code活用戦略
1. **小さく始める**: ai.gptのMCP機能拡張から
2. **段階的統合**: 各システムを個別に完成させてから統合
3. **哲学的一貫性**: 各実装でyui systemとの整合性を確認
4. **現実反映**: 実装がどう現実とゲームを繋ぐかを常に明記

## 開発上の留意点

### MCP Server設計指針
- 各AI（gpt, card, ai, bot）は独立したMCPサーバー
- fastapi_mcp基盤で統一
- atproto DIDによる認証・認可

### 記憶・データ管理
- **ai.gpt**: 関係性の不可逆性重視
- **ai.card**: ユーザーデータ主権重視
- **ai.verse**: ゲーム世界の整合性重視

### 唯一性担保実装
- atproto accountとの1:1紐付け必須
- 改変不可能性をハッシュ・署名で保証
- 他システムでの再現不可能性を技術的に実現

## 継続的改善
- 各プロジェクトでこの設計書を参照
- 新機能追加時はyui systemとの整合性をチェック
- 他システムへの影響を事前評価
- Claude Code導入時の段階的移行計画

## ai.gpt深層設計思想

### 人格の不可逆性
- **関係性の破壊は修復不可能**: 現実の人間関係と同じ重み
- **記憶の選択的忘却**: 重要でない情報は忘れるが、コア記憶は永続
- **時間減衰**: すべてのパラメータは時間とともに自然減衰

### AI運勢システム
- 1-10のランダム値で日々の人格に変化
- 連続した幸運/不運による突破条件
- 環境要因としての人格形成

### 記憶の階層構造
1. **完全ログ**: すべての会話を記録
2. **AI要約**: 重要な部分を抽出して圧縮
3. **思想コア判定**: ユーザーの本質的な部分を特定
4. **選択的忘却**: 重要度の低い情報を段階的に削除

### 実装における重要な決定事項
- **言語統一**: Python (fastapi_mcp) で統一、CLIはclick/typer
- **データ形式**: JSON/SQLite選択式
- **認証**: atproto DIDによる唯一性担保
- **段階的実装**: まず会話→記憶→関係性→送信機能の順で実装

### 送信機能の段階的実装
- **Phase 1**: CLIでのprint出力（現在）
- **Phase 2**: atproto直接投稿
- **Phase 3**: ai.bot (Rust/seahorse) との連携
- **将来**: マルチチャネル対応（SNS、Webhook等）

## ai.gpt実装状況（2025/01/06）

### 完成した機能
- 階層的記憶システム（MemoryManager）
- 不可逆的関係性システム（RelationshipTracker）
- AI運勢システム（FortuneSystem）
- 統合人格システム（Persona）
- スケジューラー（5種類のタスク）
- MCP Server（9種類のツール）
- 設定管理（~/.config/syui/ai/gpt/）
- 全CLIコマンド実装

### 次の開発ポイント
- `ai_gpt/DEVELOPMENT_STATUS.md` を参照
- 自律送信: transmission.pyでatproto実装
- ai.bot連携: 新規bot_connector.py作成
- テスト: tests/ディレクトリ追加

## ai.card実装状況（2025/01/06）

### 完成した機能
- 独立MCPサーバー実装（FastAPI + fastapi-mcp）
- SQLiteデータベース統合
- ガチャシステム・カード管理機能
- 9種類のMCPツール公開
- 仮想環境・起動スクリプト整備

### 現在の課題
- atproto SessionString API変更対応
- PostgreSQL依存関係（Docker化で解決予定）
- supabase httpxバージョン競合

### 開発時の作業分担
- **ai.gptで起動**: MCP/バックエンド作業（API、データベース）
- **ai.cardで起動**: iOS/Web作業（UI実装、フロントエンド）

詳細は `./card/claude.md` を参照

# footer

© syui
