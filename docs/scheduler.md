# スケジューラーガイド

## 概要

スケジューラーは、AIの自律的な動作を実現するための中核機能です。定期的なタスクを設定し、バックグラウンドで実行できます。

## タスクタイプ

### transmission_check
関係性が閾値を超えたユーザーへの自動送信をチェックします。

```bash
# 30分ごとにチェック
ai-gpt schedule add transmission_check "30m" --provider ollama --model qwen2.5
```

### maintenance
日次メンテナンスを実行します：
- 記憶の忘却処理
- コア記憶の判定
- 関係性パラメータの整理

```bash
# 毎日午前3時に実行
ai-gpt schedule add maintenance "0 3 * * *"
```

### fortune_update
AI運勢を更新します（通常は自動的に更新されます）。

```bash
# 毎日午前0時に強制更新
ai-gpt schedule add fortune_update "0 0 * * *"
```

### relationship_decay
時間経過による関係性の自然減衰を適用します。

```bash
# 1時間ごとに減衰処理
ai-gpt schedule add relationship_decay "1h"
```

### memory_summary
蓄積された記憶から要約を作成します。

```bash
# 週に1回、日曜日に実行
ai-gpt schedule add memory_summary "0 0 * * SUN"
```

## スケジュール形式

### Cron形式

標準的なcron式を使用できます：

```
┌───────────── 分 (0 - 59)
│ ┌───────────── 時 (0 - 23)
│ │ ┌───────────── 日 (1 - 31)
│ │ │ ┌───────────── 月 (1 - 12)
│ │ │ │ ┌───────────── 曜日 (0 - 6) (日曜日 = 0)
│ │ │ │ │
* * * * *
```

例：
- `"0 */6 * * *"` - 6時間ごと
- `"0 9 * * MON-FRI"` - 平日の午前9時
- `"*/15 * * * *"` - 15分ごと

### インターバル形式

シンプルな間隔指定：
- `"30s"` - 30秒ごと
- `"5m"` - 5分ごと
- `"2h"` - 2時間ごと
- `"1d"` - 1日ごと

## 実践例

### 基本的な自律AI設定

```bash
# 1. 30分ごとに送信チェック
ai-gpt schedule add transmission_check "30m"

# 2. 1日1回メンテナンス
ai-gpt schedule add maintenance "0 3 * * *"

# 3. 2時間ごとに関係性減衰
ai-gpt schedule add relationship_decay "2h"

# 4. 週1回記憶要約
ai-gpt schedule add memory_summary "0 0 * * MON"

# スケジューラーを起動
ai-gpt schedule run
```

### タスク管理

```bash
# タスク一覧を確認
ai-gpt schedule list

# タスクを一時停止
ai-gpt schedule disable --task-id transmission_check_1234567890

# タスクを再開
ai-gpt schedule enable --task-id transmission_check_1234567890

# 不要なタスクを削除
ai-gpt schedule remove --task-id old_task_123
```

## デーモン化

### systemdサービスとして実行

`/etc/systemd/system/ai-gpt-scheduler.service`:

```ini
[Unit]
Description=ai.gpt Scheduler
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser
ExecStart=/usr/local/bin/ai-gpt schedule run
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
# サービスを有効化
sudo systemctl enable ai-gpt-scheduler
sudo systemctl start ai-gpt-scheduler
```

### tmux/screenでバックグラウンド実行

```bash
# tmuxセッションを作成
tmux new -s ai-gpt-scheduler

# スケジューラーを起動
ai-gpt schedule run

# セッションから離脱 (Ctrl+B, D)
```

## トラブルシューティング

### タスクが実行されない

1. スケジューラーが起動しているか確認
2. タスクが有効になっているか確認：`ai-gpt schedule list`
3. ログを確認（将来実装予定）

### 重複実行を防ぐ

同じタスクタイプを複数回追加しないよう注意してください。必要に応じて古いタスクを削除してから新しいタスクを追加します。