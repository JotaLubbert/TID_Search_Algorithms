from comun import *

COLS = ["expansions", "close_bytes", "close_nodes", "execution_time"]
OTROS = [e for e in ESTRUCTURAS if e != BASE]


def celdas(mapa):
    # ancho x alto leídos del encabezado del .map
    for carpeta_mapas in ["maps", "maze_maps"]:
        try:
            with open(f"{carpeta_mapas}/{mapa}", encoding="utf-8") as fh:
                l = [fh.readline() for _ in range(3)]
            return int(l[1].split()[1]) * int(l[2].split()[1])
        except Exception:
            continue
    return float("nan")


filas = []
for nombre, d in cargar(COLS):
    mapa = nombre.replace(".scen.tsv", "")
    a = d[BASE]
    fila = {
        "mapa": mapa.replace(".map", ""),
        "celdas": celdas(mapa),
        "n": len(a),
        "expansiones": a["expansions"].mean(),
        f"{BASE} B/nodo": a["close_bytes"].sum() / a["close_nodes"].sum(),
    }
    for e in OTROS:
        b = d[e]
        fila[f"{e} tiempo %"] = (b["execution_time"].sum() / a["execution_time"].sum() - 1) * 100
        fila[f"{e} B/nodo"] = b["close_bytes"].sum() / b["close_nodes"].sum()
    filas.append(fila)

if not filas:
    sys.exit("Sin datos para este grupo.")

df = pd.DataFrame(filas).sort_values("judy tiempo %", ascending=False)
pd.set_option("display.width", 300)
pd.set_option("display.max_columns", None)
print(f"Grupo: {GRUPO}  (tiempo % = estructura frente a {BASE}; negativo = mejor)\n")
print(df.round(1).to_string(index=False))

print("\nCorrelación (Spearman) del tiempo % de Judy con:")
for c in ["celdas", "expansiones", "judy B/nodo"]:
    # Spearman = correlación de Pearson entre los rangos (no necesita scipy)
    print(f"  {c}: {df['judy tiempo %'].rank().corr(df[c].rank()):.2f}")