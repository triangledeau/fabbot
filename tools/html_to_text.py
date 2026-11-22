"""Convertit les pages HTML de rules.fabtcg.com en texte brut (un .txt par .html)."""
import html
import re
import sys

for fn in sys.argv[1:]:
    with open(fn, encoding="utf-8") as fh:
        t = fh.read()
    m = re.search(r"(?s)<article.*?</article>", t)
    if m:
        t = m.group(0)
    t = re.sub(r"(?s)<(script|style).*?</\1>", "", t)
    t = re.sub(r"<br\s*/?>|</(p|li|h\d|div|tr)>", "\n", t)
    t = html.unescape(re.sub(r"<[^>]+>", "", t)).replace("§", "")
    t = re.sub(r"\n\s*\n+", "\n", t)
    with open(fn[:-5] + ".txt", "w", encoding="utf-8") as fh:
        fh.write(t)
