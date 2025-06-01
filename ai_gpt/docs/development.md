# 開発者向けガイド

## アーキテクチャ

### ディレクトリ構造

```
ai_gpt/
├── src/ai_gpt/
│   ├── __init__.py
│   ├── models.py        # データモデル定義
│   ├── memory.py        # 記憶管理システム
│   ├── relationship.py  # 関係性トラッカー
│   ├── fortune.py       # AI運勢システム
│   ├── persona.py       # 統合人格システム
│   ├── transmission.py  # 送信コントローラー
│   ├── scheduler.py     # スケジューラー
│   ├── config.py        # 設定管理
│   ├── ai_provider.py   # AI統合（Ollama/OpenAI）
│   ├── mcp_server.py    # MCP Server実装
│   └── cli.py          # CLIインターフェース
├── docs/               # ドキュメント
├── tests/              # テスト
└── pyproject.toml      # プロジェクト設定
```

### 主要コンポーネント

#### MemoryManager
階層的記憶システムの実装。会話を記録し、要約・コア判定・忘却を管理。

```python
memory = MemoryManager(data_dir)
memory.add_conversation(conversation)
memory.summarize_memories(user_id)
memory.identify_core_memories()
memory.apply_forgetting()
```

#### RelationshipTracker
ユーザーとの関係性を追跡。不可逆的なダメージと時間減衰を実装。

```python
tracker = RelationshipTracker(data_dir)
relationship = tracker.update_interaction(user_id, delta)
tracker.apply_time_decay()
```

#### Persona
すべてのコンポーネントを統合し、一貫した人格を提供。

```python
persona = Persona(data_dir)
response, delta = persona.process_interaction(user_id, message)
state = persona.get_current_state()
```

## 拡張方法

### 新しいAIプロバイダーの追加

1. `ai_provider.py`に新しいプロバイダークラスを作成：

```python
class CustomProvider:
    async def generate_response(
        self,
        prompt: str,
        persona_state: PersonaState,
        memories: List[Memory],
        system_prompt: Optional[str] = None
    ) -> str:
        # 実装
        pass
```

2. `create_ai_provider`関数に追加：

```python
def create_ai_provider(provider: str, model: str, **kwargs):
    if provider == "custom":
        return CustomProvider(model=model, **kwargs)
    # ...
```

### 新しいスケジュールタスクの追加

1. `TaskType`enumに追加：

```python
class TaskType(str, Enum):
    CUSTOM_TASK = "custom_task"
```

2. ハンドラーを実装：

```python
async def _handle_custom_task(self, task: ScheduledTask):
    # タスクの実装
    pass
```

3. `task_handlers`に登録：

```python
self.task_handlers[TaskType.CUSTOM_TASK] = self._handle_custom_task
```

### 新しいMCPツールの追加

`mcp_server.py`の`_register_tools`メソッドに追加：

```python
@self.server.tool("custom_tool")
async def custom_tool(param1: str, param2: int) -> Dict[str, Any]:
    """カスタムツールの説明"""
    # 実装
    return {"result": "value"}
```

## テスト

```bash
# テストの実行（将来実装）
pytest tests/

# 特定のテスト
pytest tests/test_memory.py
```

## デバッグ

### ログレベルの設定

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

### データファイルの直接確認

```bash
# 関係性データを確認
cat ~/.config/aigpt/data/relationships.json | jq

# 記憶データを確認
cat ~/.config/aigpt/data/memories.json | jq
```

## 貢献方法

1. フォークする
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## 設計原則

1. **不可逆性**: 一度失われた関係性は回復しない
2. **階層性**: 記憶は重要度によって階層化される
3. **自律性**: AIは関係性に基づいて自発的に行動する
4. **唯一性**: 各ユーザーとの関係は唯一無二

## ライセンス

MIT License