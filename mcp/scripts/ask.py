import os
import json
import httpx
import openai

from context_loader import load_context_from_repo
from prompt_template import PROMPT_TEMPLATE

PROVIDER = os.getenv("PROVIDER", "ollama")  # "ollama" or "openai"

# Ollama用
OLLAMA_HOST = os.getenv("OLLAMA_HOST", "http://localhost:11434")
OLLAMA_URL = f"{OLLAMA_HOST}/api/generate"
OLLAMA_MODEL = os.getenv("MODEL", "syui/ai")

# OpenAI用
OPENAI_BASE = os.getenv("OPENAI_API_BASE", "https://api.openai.com/v1")
OPENAI_KEY = os.getenv("OPENAI_API_KEY", "")
OPENAI_MODEL = os.getenv("MODEL", "gpt-4o-mini")

def ask_question(question, repo_path="."):
    context = load_context_from_repo(repo_path)
    prompt = PROMPT_TEMPLATE.format(context=context[:10000], question=question)

    if PROVIDER == "ollama":
        payload = {
            "model": OLLAMA_MODEL,
            "prompt": prompt,
            "stream": False
        }
        response = httpx.post(OLLAMA_URL, json=payload, timeout=60.0)
        result = response.json()
        return result.get("response", "返答がありませんでした。")

    elif PROVIDER == "openai":
        import openai
        openai.api_key = OPENAI_KEY
        openai.api_base = OPENAI_BASE

        client = openai.OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
        response = client.chat.completions.create(
            model=OPENAI_MODEL,
            messages=[{"role": "user", "content": prompt}]
        )
        return response.choices[0].message.content

    else:
        return f"❌ 未知のプロバイダです: {PROVIDER}"


if __name__ == "__main__":
    import sys
    question = " ".join(sys.argv[1:])
    answer = ask_question(question)
    print("\n🧠 回答:\n", answer)
