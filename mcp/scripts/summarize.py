# scripts/summarize.py
import json
from datetime import datetime
from config import MEMORY_DIR, SUMMARY_DIR, load_config
import requests

def load_memory(date_str):
    path = MEMORY_DIR / f"{date_str}.json"
    if not path.exists():
        print(f"⚠️ メモリファイルが見つかりません: {path}")
        return None
    with open(path, "r") as f:
        return json.load(f)

def save_summary(date_str, content):
    SUMMARY_DIR.mkdir(parents=True, exist_ok=True)
    path = SUMMARY_DIR / f"{date_str}_summary.json"
    with open(path, "w") as f:
        json.dump(content, f, indent=2, ensure_ascii=False)
    print(f"✅ 要約を保存しました: {path}")

def build_prompt(logs):
    messages = [
        {"role": "system", "content": "あなたは要約AIです。以下の会話ログを要約してください。"},
        {"role": "user", "content": "\n".join(f"{entry['sender']}: {entry['message']}" for entry in logs)}
    ]
    return messages

def summarize_with_llm(messages):
    cfg = load_config()
    if cfg["provider"] == "openai":
        headers = {
            "Authorization": f"Bearer {cfg['api_key']}",
            "Content-Type": "application/json",
        }
        payload = {
            "model": cfg["model"],
            "messages": messages,
            "temperature": 0.7
        }
        response = requests.post(cfg["url"], headers=headers, json=payload)
        response.raise_for_status()
        return response.json()["choices"][0]["message"]["content"]

    elif cfg["provider"] == "ollama":
        payload = {
            "model": cfg["model"],
            "prompt": "\n".join(m["content"] for m in messages),
            "stream": False,
        }
        response = requests.post(cfg["url"], json=payload)
        response.raise_for_status()
        return response.json()["response"]

    else:
        raise ValueError(f"Unsupported provider: {cfg['provider']}")

def main():
    date_str = datetime.now().strftime("%Y-%m-%d")
    logs = load_memory(date_str)
    if not logs:
        return

    prompt_messages = build_prompt(logs)
    summary_text = summarize_with_llm(prompt_messages)

    summary = {
        "date": date_str,
        "summary": summary_text,
        "total_messages": len(logs)
    }

    save_summary(date_str, summary)

if __name__ == "__main__":
    main()
