import pandas as pd
from pathlib import Path

estructuras = ["hashmap", "robin_hood", "judy"]
cols = ["execution_time", "closed_get_ns", "closed_insert_ns", "closed_gets", "closed_inserts"]
base = Path("test_result")

tot = {e: dict.fromkeys(cols, 0) for e in estructuras}
escenarios = 0
for f in sorted((base / "hashmap").iterdir()):
    datos = {}
    for e in estructuras:
        g = base / e / f.name
        if g.exists():
            datos[e] = pd.read_csv(g, sep="\t", usecols=cols)
    if len(datos) < len(estructuras):
        continue  # el mapa no está en las tres estructuras
    n = min(len(d) for d in datos.values())  # mismas filas para todas
    escenarios += n
    for e, d in datos.items():
        for c in cols:
            tot[e][c] += int(d[c][:n].sum())

print(f"Escenarios sumados: {escenarios:,}\n")
print(f"{'estructura':<11} {'get (s)':>9} {'insert (s)':>11} {'Closed (s)':>11} {'ns/get':>8} {'ns/insert':>10} {'% de A*':>8} {'vs hashmap':>11}")
base_closed = (tot["hashmap"]["closed_get_ns"] + tot["hashmap"]["closed_insert_ns"]) / 1e9
for e in estructuras:
    t = tot[e]
    get_s = t["closed_get_ns"] / 1e9
    ins_s = t["closed_insert_ns"] / 1e9
    closed_s = get_s + ins_s
    ns_get = t["closed_get_ns"] / t["closed_gets"]
    ns_ins = t["closed_insert_ns"] / t["closed_inserts"]
    share = closed_s / (t["execution_time"] / 1e6) * 100   # execution_time está en microsegundos
    vs = "base" if e == "hashmap" else f"{(closed_s / base_closed - 1) * 100:+.1f}%"
    print(f"{e:<11} {get_s:>9.2f} {ins_s:>11.2f} {closed_s:>11.2f} {ns_get:>8.1f} {ns_ins:>10.1f} {share:>7.1f}% {vs:>11}")