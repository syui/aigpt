import httpx
import os
import json
from context_loader import load_context_from_repo
from prompt_template import PROMPT_TEMPLATE

OLLAMA_HOST = os.getenv("OLLAMA_HOST", "http://localhost:11434")
OLLAMA_URL = f"{OLLAMA_HOST}/api/generate"
OLLAMA_MODEL = os.getenv("OLLAMA_MODEL", "syui/ai")

def ask_question(question, repo_path="."):
    context = load_context_from_repo(repo_path)
    prompt = PROMPT_TEMPLATE.format(context=context[:10000], question=question)

    payload = {
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": False
    }

    #response = httpx.post(OLLAMA_URL, json=payload)
    response = httpx.post(OLLAMA_URL, json=payload, timeout=60.0)
    result = response.json()
    return result.get("response", "返答がありませんでした。")

if __name__ == "__main__":
    import sys
    question = " ".join(sys.argv[1:])
    answer = ask_question(question)
    print("\n🧠 回答:\n", answer)
