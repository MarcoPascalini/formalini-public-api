from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
missing = []
for page in root.rglob("*.md"):
    if any(part in {".git", ".tools", "obj", "bin"} for part in page.parts):
        continue
    for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", page.read_text(encoding="utf-8")):
        path = target.split("#", 1)[0]
        if not path or "://" in path or path.startswith("mailto:"):
            continue
        if not (page.parent / path).exists():
            missing.append(f"{page.relative_to(root)}: {target}")

if missing:
    print("Broken local Markdown links:\n" + "\n".join(missing))
    sys.exit(1)
print("Local Markdown links resolve.")
