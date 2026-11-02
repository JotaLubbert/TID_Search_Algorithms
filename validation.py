import numpy as np
import pandas as pd
from pathlib import Path

estructuras = ["hashmap", "hashmap_hash_propio", "robin_hood", "robin_hood_hash_propio", "judy"]

for e in estructuras:
    carpeta = Path("test_result") / e
    if not carpeta.exists():
        continue
    total = ok_abs = ok_rel = 0
    ejemplos = []
    for f in sorted(carpeta.iterdir()):
        d = pd.read_csv(f, sep="\t", usecols=["expected_distance", "actual_distance"])
        esperado = d["expected_distance"].to_numpy()
        obtenido = d["actual_distance"].to_numpy()
        bien = np.isclose(obtenido, esperado, rtol=0, atol=1e-6)
        total += len(d)
        ok_abs += int(bien.sum())
        ok_rel += int(np.isclose(obtenido, esperado, rtol=1e-4, atol=0).sum())
        ejemplos += [(f.name, int(i), esperado[i], obtenido[i]) for i in np.flatnonzero(~bien)[:2]]
    print(f"{e:<24} escenarios {total:,}  coinciden (|dif| ≤ 1e-6): {ok_abs:,}  (rtol 1e-4: {ok_rel:,})")
    for m in ejemplos[:5]:
        print("    ejemplo:", m)