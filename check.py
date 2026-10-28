import pandas as pd
from pathlib import Path

base = Path("test_result/hashmap")
cols = ["actual_distance", "path_len", "expansions", "generated", "path"]

for other in ["robin_hood", "judy"]:
    print(f"--- hashmap vs {other} ---")
    for f in sorted(base.iterdir()):
        g = Path("test_result") / other / f.name
        if not g.exists():
            continue
        a = pd.read_csv(f, sep="\t")
        b = pd.read_csv(g, sep="\t")
        n = min(len(a), len(b))
        bad = [c for c in cols if not (a[c][:n].values == b[c][:n].values).all()]
        if not bad:
            print(f.name, f"n={n}", "OK")
        else:
            c = bad[0]
            row = (a[c][:n].values != b[c][:n].values).argmax()
            print(f.name, f"n={n}", f"DIFIEREN en {bad}, primera fila distinta: {row}")