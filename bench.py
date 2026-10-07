import pandas as pd
from pathlib import Path

base = Path("test_result/hashmap")
others = ["robin_hood", "judy"]
cols = ["close_bytes", "execution_time"]  # execution_time está en microsegundos

for other in others:
    print(f"\n=== {other} - hashmap (negativo = mejor) ===")
    diffs = []
    bases = []
    for f in sorted(base.iterdir()):
        g = Path("test_result") / other / f.name
        if not g.exists():
            continue
        a = pd.read_csv(f, sep="\t")
        b = pd.read_csv(g, sep="\t")
        n = min(len(a), len(b))
        a = a[cols][:n].reset_index(drop=True)
        b = b[cols][:n].reset_index(drop=True)
        d = b - a
        pct = d.mean() / a.mean() * 100
        print(f"{f.name:35s} n={n:5d}  "
              f"close_bytes {d['close_bytes'].mean():+14.1f} ({pct['close_bytes']:+6.1f}%)  "
              f"tiempo {d['execution_time'].mean():+12.1f} us ({pct['execution_time']:+6.1f}%)")
        diffs.append(d)
        bases.append(a)
    if diffs:
        total = pd.concat(diffs)
        total_base = pd.concat(bases)
        gpct = total.mean() / total_base.mean() * 100
        print(f"{'GLOBAL':35s} n={len(total):5d}  "
              f"close_bytes {total['close_bytes'].mean():+14.1f} ({gpct['close_bytes']:+6.1f}%)  "
              f"tiempo {total['execution_time'].mean():+12.1f} us ({gpct['execution_time']:+6.1f}%)")
