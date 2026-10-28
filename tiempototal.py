import pandas as pd
from pathlib import Path

estructuras = ["hashmap", "robin_hood", "judy"]
base = Path("test_result")

totales = {e: 0 for e in estructuras}   # microsegundos
escenarios = 0
por_mapa = []

for f in sorted((base / "hashmap").iterdir()):
    datos = {}
    for e in estructuras:
        g = base / e / f.name
        if g.exists():
            datos[e] = pd.read_csv(g, sep="\t", usecols=["execution_time"])["execution_time"]
    if len(datos) < len(estructuras):
        continue  # el mapa no está en las tres estructuras: no se compara
    n = min(len(s) for s in datos.values())  # mismas filas para todas, aunque alguna esté incompleta
    fila = {"mapa": f.name.replace(".map.scen.tsv", ""), "escenarios": n}
    for e in estructuras:
        t = int(datos[e][:n].sum())
        totales[e] += t
        fila[e] = t
    escenarios += n
    por_mapa.append(fila)

print(f"Escenarios sumados: {escenarios:,}\n")
print(f"{'estructura':<12} {'tiempo total':>14} {'vs hashmap':>14} {'diferencia':>14}")
base_t = totales["hashmap"]
for e in estructuras:
    seg = totales[e] / 1e6
    if e == "hashmap":
        print(f"{e:<12} {seg:>12,.2f} s {'base':>14} {'':>14}")
    else:
        dif = (totales[e] - base_t) / 1e6
        pct = (totales[e] / base_t - 1) * 100
        print(f"{e:<12} {seg:>12,.2f} s {pct:>+13.1f}% {dif:>+12,.2f} s")

print("\nPor mapa (segundos):")
df = pd.DataFrame(por_mapa)
for e in estructuras:
    df[e] = df[e] / 1e6
print(df.round(2).to_string(index=False))