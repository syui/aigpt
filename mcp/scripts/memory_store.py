# scripts/memory_store.py
import json
from pathlib import Path
from config import MEMORY_DIR
from datetime import datetime, timezone

def load_logs(date_str=None):
    if date_str is None:
        date_str = datetime.now().strftime("%Y-%m-%d")
    path = MEMORY_DIR / f"{date_str}.json"
    if path.exists():
        with open(path, "r") as f:
            return json.load(f)
    return []

def save_message(sender, message):
    date_str = datetime.now().strftime("%Y-%m-%d")
    path = MEMORY_DIR / f"{date_str}.json"
    logs = load_logs(date_str)
    now = datetime.now(timezone.utc).isoformat()
    logs.append({"timestamp": now, "sender": sender, "message": message})
    with open(path, "w") as f:
        json.dump(logs, f, indent=2, ensure_ascii=False)

def search_memory(query: str):
    from glob import glob
    all_logs = []
    pattern = re.compile(re.escape(query), re.IGNORECASE)

    for file_path in sorted(MEMORY_DIR.glob("*.json")):
        with open(file_path, "r") as f:
            logs = json.load(f)
            matched = [entry for entry in logs if pattern.search(entry["message"])]
            all_logs.extend(matched)

    return all_logs[-5:]

# scripts/memory_store.py
import json
from datetime import datetime
from pathlib import Path
from config import MEMORY_DIR

# ログを読み込む（指定日または当日）
def load_logs(date_str=None):
    if date_str is None:
        date_str = datetime.now().strftime("%Y-%m-%d")
    path = MEMORY_DIR / f"{date_str}.json"
    if path.exists():
        with open(path, "r") as f:
            return json.load(f)
    return []

# メッセージを保存する
def save_message(sender, message):
    date_str = datetime.now().strftime("%Y-%m-%d")
    path = MEMORY_DIR / f"{date_str}.json"
    logs = load_logs(date_str)
    #now = datetime.utcnow().isoformat() + "Z"
    now = datetime.now(timezone.utc).isoformat()
    logs.append({"timestamp": now, "sender": sender, "message": message})
    with open(path, "w") as f:
        json.dump(logs, f, indent=2, ensure_ascii=False)

def search_memory(query: str):
    from glob import glob
    all_logs = []
    for file_path in sorted(MEMORY_DIR.glob("*.json")):
        with open(file_path, "r") as f:
            logs = json.load(f)
            matched = [
                entry for entry in logs 
                if entry["sender"] == "user" and query in entry["message"]
            ]
            all_logs.extend(matched)
    return all_logs[-5:]  # 最新5件だけ返す
def search_memory(query: str):
    from glob import glob
    all_logs = []
    seen_messages = set()  # すでに見たメッセージを保持

    for file_path in sorted(MEMORY_DIR.glob("*.json")):
        with open(file_path, "r") as f:
            logs = json.load(f)
            for entry in logs:
                if entry["sender"] == "user" and query in entry["message"]:
                    # すでに同じメッセージが結果に含まれていなければ追加
                    if entry["message"] not in seen_messages:
                        all_logs.append(entry)
                        seen_messages.add(entry["message"])

    return all_logs[-5:]  # 最新5件だけ返す
