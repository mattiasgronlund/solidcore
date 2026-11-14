import json, sys, os
exp, apple, cr, out = sys.argv[1:5]
def load(p):
    d = {}
    if not os.path.exists(p): return d
    text, i, dec = open(p).read(), 0, json.JSONDecoder()
    while True:
        while i < len(text) and text[i].isspace(): i += 1
        if i >= len(text): break
        start = i
        o, i = dec.raw_decode(text, i)
        if isinstance(o, dict) and "case" in o: d[o["case"]] = text[start:i]
    return d
def beyond(a, b):
    # True when some number differs by more than 1e-9 relative, or anything non-numeric differs
    if isinstance(a, bool) or isinstance(b, bool): return a != b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return abs(a - b) > 1e-9 * max(abs(a), abs(b))
    if isinstance(a, str) or isinstance(b, str) or a is None or b is None: return a != b
    if isinstance(a, list) and isinstance(b, list):
        return len(a) != len(b) or any(beyond(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() != b.keys() or any(beyond(a[k], b[k]) for k in a)
    return a != b
n = nb = 0
with open(out, "w") as f:
    for name in sorted(os.listdir(exp)):
        g, a, c = load(f"{exp}/{name}"), load(f"{apple}/{name}"), load(f"{cr}/{name}")
        for case in sorted(set(g) | set(c)):
            if g.get(case) == c.get(case): continue
            gv = json.loads(g[case]) if case in g else None
            cv = json.loads(c[case]) if case in c else None
            b = gv is None or cv is None or beyond(gv, cv)
            n += 1; nb += b
            f.write(json.dumps({"unit": name[:-6], "case": case, "beyond_1e-9_relative": b,
                "golden": gv, "apple_libm": json.loads(a[case]) if case in a else None, "cr": cv}) + "\n")
print(f"{n} cases differ bit for bit, {nb} beyond 1e-9 relative (no abs floors)")
