import os

def load_context_from_repo(repo_path: str, extensions={".rs", ".toml", ".md"}) -> str:
    context = ""
    for root, dirs, files in os.walk(repo_path):
        for file in files:
            if any(file.endswith(ext) for ext in extensions):
                with open(os.path.join(root, file), "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                    context += f"\n\n# FILE: {os.path.join(root, file)}\n{content}"
    return context
