# 使い方ガイド 📖

## 🚀 aigpt の起動方法

### 1. ビルド

```bash
# ローカル環境で実行
cd /path/to/aigpt
cargo build --release --features ai-analysis
```

### 2. Claude API キーの設定

```bash
# 環境変数で設定
export ANTHROPIC_API_KEY=sk-ant-...

# モデルを指定（オプション）
export ANTHROPIC_MODEL=claude-3-5-sonnet-20241022  # デフォルトは haiku
```

### 3. MCPサーバーとして起動

```bash
# 起動
./target/release/aigpt server

# またはAPI キーを直接指定
ANTHROPIC_API_KEY=sk-ant-... ./target/release/aigpt server
```

---

## 🎮 Claude Code での使い方

### 設定方法

#### 方法1: コマンドで追加（推奨！）

```bash
claude mcp add aigpt /home/user/aigpt/target/release/aigpt server
```

#### 方法2: 設定ファイルを直接編集

`~/.config/claude-code/config.json` に追加：

```json
{
  "mcpServers": {
    "aigpt": {
      "command": "/home/user/aigpt/target/release/aigpt",
      "args": ["server"]
    }
  }
}
```

**注意**: 環境変数 (env) は不要です！完全にローカルで動作します。

### Claude Code を再起動

設定後、Claude Code を再起動すると、11個のツールが使えるようになります。

---

## 💬 実際の使用例

### 例1: メモリを作成

**あなた（Claude Codeで話しかける）:**
> 「今日、新しいAIシステムのアイデアを思いついた」というメモリを作成して

**Claude Code の動作:**
1. `create_memory_with_ai` ツールを自動で呼び出す
2. Claude API があなたの入力を解釈
3. 4つの心スコア（感情、関連性、新規性、実用性）を計算
4. priority_score (0.0-1.0) を算出
5. ゲーム風の結果を表示

**結果の表示:**
```
╔══════════════════════════════════════╗
║    🎲 メモリースコア判定          ║
╚══════════════════════════════════════╝

🟣 EPIC 85点
💡 あなたは【革新者】タイプ！

💕 好感度: ❤️❤️❤️❤️❤️🤍🤍🤍🤍🤍 42.5%
💎 XP獲得: +850 XP

📊 スコア内訳:
  感情的インパクト: ████████░░ 20%
  あなたへの関連性: ████████░░ 20%
  新規性・独自性: █████████░ 22.5%
  実用性・有用性: █████████░ 22.5%
```

### 例2: コンパニオンを作成

**あなた:**
> 「エミリー」という名前のエネルギッシュなコンパニオンを作成して

**結果:**
```
╔══════════════════════════════════════╗
║  💕 エミリー のプロフィール      ║
╚══════════════════════════════════════╝

⚡ 性格: エネルギッシュで冒険好き
「新しいことに挑戦するのが大好き！」

🏆 関係レベル: Lv.1
💕 好感度: 🤍🤍🤍🤍🤍🤍🤍🤍🤍🤍 0%
🤝 信頼度: ░░░░░░░░░░ 0/100
```

### 例3: コンパニオンに反応してもらう

**あなた:**
> 先ほど作ったメモリにエミリーを反応させて

**結果:**
```
⚡ エミリー:
「わあ！新しいAIシステムのアイデアって
すごくワクワクするね！💡
あなたの創造力、本当に素敵だと思う！」

💕 好感度変化: 0% → 80.75% ⬆️ +80.75%
🎊 ボーナス: ⚡相性抜群！ (+95%)
💎 XP獲得: +850 XP
🏆 レベルアップ: Lv.1 → Lv.9
```

### 例4: ランキングを見る

**あなた:**
> メモリをランキング順に表示して

**結果:**
```
╔══════════════════════════════════════╗
║    🏆 メモリーランキング TOP10    ║
╚══════════════════════════════════════╝

1. 🟡 LEGENDARY 95点 - 「AI哲学について...」
2. 🟣 EPIC 85点 - 「新しいシステムのアイデア」
3. 🔵 RARE 75点 - 「プロジェクトの進捗」
...
```

---

## 📊 結果の見方

### レアリティシステム
- 🟡 **LEGENDARY** (90-100点): 伝説級の記憶
- 🟣 **EPIC** (80-89点): エピック級の記憶
- 🔵 **RARE** (60-79点): レアな記憶
- 🟢 **UNCOMMON** (40-59点): まあまあの記憶
- ⚪ **COMMON** (0-39点): 日常的な記憶

### 診断タイプ（あなたの個性）
- 💡 **革新者**: 創造性と実用性が高い
- 🧠 **哲学者**: 感情と新規性が高い
- 🎯 **実務家**: 実用性と関連性が高い
- ✨ **夢想家**: 新規性と感情が高い
- 📊 **分析家**: バランス型

### コンパニオン性格
- ⚡ **Energetic**: 革新者と相性95%
- 📚 **Intellectual**: 哲学者と相性95%
- 🎯 **Practical**: 実務家と相性95%
- 🌙 **Dreamy**: 夢想家と相性95%
- ⚖️ **Balanced**: 分析家と相性95%

---

## 💾 データの保存場所

```
~/.config/syui/ai/gpt/memory.json
```

このファイルに、すべてのメモリとコンパニオン情報が保存されます。

**データ形式:**
```json
{
  "memories": {
    "uuid-1234": {
      "id": "uuid-1234",
      "content": "元の入力",
      "interpreted_content": "Claude の解釈",
      "priority_score": 0.85,
      "user_context": null,
      "created_at": "2025-11-05T...",
      "updated_at": "2025-11-05T..."
    }
  },
  "conversations": {}
}
```

---

## 🎯 利用可能なMCPツール（11個）

### 基本ツール
1. **create_memory** - シンプルなメモリ作成
2. **search_memories** - メモリ検索
3. **update_memory** - メモリ更新
4. **delete_memory** - メモリ削除
5. **list_conversations** - 会話一覧

### AI機能ツール 🎮
6. **create_memory_with_ai** - AI解釈＋ゲーム結果
7. **list_memories_by_priority** - ランキング表示
8. **daily_challenge** - デイリークエスト

### コンパニオンツール 💕
9. **create_companion** - コンパニオン作成
10. **companion_react** - メモリへの反応
11. **companion_profile** - プロフィール表示

---

## ⚙️ トラブルシューティング

### ビルドできない
```bash
# 依存関係を更新
cargo clean
cargo update
cargo build --release --features ai-analysis
```

### Claude API エラー
```bash
# APIキーを確認
echo $ANTHROPIC_API_KEY

# 正しく設定
export ANTHROPIC_API_KEY=sk-ant-...
```

### MCPサーバーが認識されない
1. Claude Code を完全に再起動
2. config.json のパスが正しいか確認
3. バイナリが存在するか確認: `ls -la /home/user/aigpt/target/release/aigpt`

### データが保存されない
```bash
# ディレクトリを確認
ls -la ~/.config/syui/ai/gpt/

# なければ手動作成
mkdir -p ~/.config/syui/ai/gpt/
```

---

## 🎉 楽しみ方のコツ

1. **毎日記録**: 日々の気づきを記録して、自分の傾向を知る
2. **タイプ診断**: どのタイプが多いか確認して、自己分析
3. **コンパニオン育成**: 好感度とレベルを上げて、絆を深める
4. **ランキング確認**: 定期的にTOP10を見て、重要な記憶を振り返る

---

## 📝 注意事項

- **APIコスト**: Claude API の使用には料金が発生します
  - Haiku: 約$0.25 / 1M tokens（入力）
  - Sonnet: 約$3.00 / 1M tokens（入力）
- **プライバシー**: メモリは Anthropic に送信されます
- **容量制限**: デフォルト100件まで（低スコアから自動削除）

---

これで aigpt を存分に楽しめます！🚀
