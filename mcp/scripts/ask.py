## scripts/ask.py
import sys
import json
import requests
from config import load_config
from datetime import datetime, timezone

def build_payload_openai(cfg, message: str):
    return {
        "model": cfg["model"],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "ask_message",
                    "description": "過去の記憶を検索します",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "検索したい語句"
                            }
                        },
                        "required": ["query"]
                    }
                }
            }
        ],
        "tool_choice": "auto",
        "messages": [
            {"role": "system", "content": "あなたは親しみやすいAIで、必要に応じて記憶から情報を検索して応答します。"},
            {"role": "user", "content": message}
        ]
    }

def build_payload_mcp(message: str):
    return {
        "tool": "ask_message",  # MCPサーバー側で定義されたツール名
        "input": {
            "message": message
        }
    }

def build_payload_openai(cfg, message: str):
    return {
        "model": cfg["model"],
        "messages": [
            {"role": "system", "content": "あなたは思いやりのあるAIです。"},
            {"role": "user", "content": message}
        ],
        "temperature": 0.7
    }

def call_mcp(cfg, message: str):
    payload = build_payload_mcp(message)
    headers = {"Content-Type": "application/json"}
    response = requests.post(cfg["url"], headers=headers, json=payload)
    response.raise_for_status()
    return response.json().get("output", {}).get("response", "❓ 応答が取得できませんでした")

def call_openai(cfg, message: str):
    # ツール定義
    tools = [
        {
            "type": "function",
            "function": {
                "name": "memory",
                "description": "記憶を検索する",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "検索する語句"
                        }
                    },
                    "required": ["query"]
                }
            }
        }
    ]

    # 最初のメッセージ送信
    payload = {
        "model": cfg["model"],
        "messages": [
            {"role": "system", "content": "あなたはAIで、必要に応じてツールmemoryを使って記憶を検索します。"},
            {"role": "user", "content": message}
        ],
        "tools": tools,
        "tool_choice": "auto"
    }

    headers = {
        "Authorization": f"Bearer {cfg['api_key']}",
        "Content-Type": "application/json",
    }

    res1 = requests.post(cfg["url"], headers=headers, json=payload)
    res1.raise_for_status()
    result = res1.json()

    # 🧠 tool_call されたか確認
    if "tool_calls" in result["choices"][0]["message"]:
        tool_call = result["choices"][0]["message"]["tool_calls"][0]
        if tool_call["function"]["name"] == "memory":
            args = json.loads(tool_call["function"]["arguments"])
            query = args.get("query", "")
            print(f"🛠️ ツール実行: memory(query='{query}')")

            # MCPエンドポイントにPOST
            memory_res = requests.post("http://127.0.0.1:5000/memory/search", json={"query": query})
            memory_json = memory_res.json()
            tool_output = memory_json.get("result", "なし")

            # tool_outputをAIに返す
            followup = {
                "model": cfg["model"],
                "messages": [
                    {"role": "system", "content": "あなたはAIで、必要に応じてツールmemoryを使って記憶を検索します。"},
                    {"role": "user", "content": message},
                    {"role": "assistant", "tool_calls": result["choices"][0]["message"]["tool_calls"]},
                    {"role": "tool", "tool_call_id": tool_call["id"], "name": "memory", "content": tool_output}
                ]
            }

            res2 = requests.post(cfg["url"], headers=headers, json=followup)
            res2.raise_for_status()
            final_response = res2.json()
            return final_response["choices"][0]["message"]["content"]
            #print(tool_output)
            #print(cfg["model"])
            #print(final_response)

    # ツール未使用 or 通常応答
    return result["choices"][0]["message"]["content"]

def call_ollama(cfg, message: str):
    payload = {
            "model": cfg["model"],
            "prompt": message,  # `prompt` → `message` にすべき（変数未定義エラー回避）
            "stream": False
            }
    headers = {"Content-Type": "application/json"}
    response = requests.post(cfg["url"], headers=headers, json=payload)
    response.raise_for_status()
    return response.json().get("response", "❌ 応答が取得できませんでした")
def main():
    if len(sys.argv) < 2:
        print("Usage: ask.py 'your message'")
        return

    message = sys.argv[1]
    cfg = load_config()

    print(f"🔍 使用プロバイダー: {cfg['provider']}")

    try:
        if cfg["provider"] == "openai":
            response = call_openai(cfg, message)
        elif cfg["provider"] == "mcp":
            response = call_mcp(cfg, message)
        elif cfg["provider"] == "ollama":
            response = call_ollama(cfg, message)
        else:
            raise ValueError(f"未対応のプロバイダー: {cfg['provider']}")

        print("💬 応答:")
        print(response)

        # ログ保存（オプション）
        save_log(message, response)

    except Exception as e:
        print(f"❌ 実行エラー: {e}")

def save_log(user_msg, ai_msg):
    from config import MEMORY_DIR
    date_str = datetime.now().strftime("%Y-%m-%d")
    path = MEMORY_DIR / f"{date_str}.json"
    path.parent.mkdir(parents=True, exist_ok=True)

    if path.exists():
        with open(path, "r") as f:
            logs = json.load(f)
    else:
        logs = []

    now = datetime.now(timezone.utc).isoformat()
    logs.append({"timestamp": now, "sender": "user", "message": user_msg})
    logs.append({"timestamp": now, "sender": "ai", "message": ai_msg})

    with open(path, "w") as f:
        json.dump(logs, f, indent=2, ensure_ascii=False)

if __name__ == "__main__":
    main()
