import pandas as pd
from pathlib import Path

base = Path("test_result/hashmap")
others = ["robin_hood", "judy"]

for other in others:
    print(f"--- hashmap vs {other} ---")
    for f in sorted(base.iterdir()):
        g = Path("test_result") / other / f.name
        if not g.exists():
            continue
        a = pd.read_csv(f, sep="\t")
        b = pd.read_csv(g, sep="\t")
        n = min(len(a), len(b))
        paths = (a["path"][:n] == b["path"][:n]).all()
        dist = (a["actual_distance"][:n] == b["actual_distance"][:n]).all()
        print(f.name, f"filas {len(a)}/{len(b)}", "OK" if paths and dist else "DIFIEREN")