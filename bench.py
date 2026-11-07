from comun import *

# Todas las comparaciones entre estructuras, a partir de los promedios por escenario
# sobre los mismos escenarios. Uso: python bench.py [grupo] [-v]
COLS = ["close_bytes", "execution_time"]

suma = {e: pd.Series(0.0, index=COLS) for e in ESTRUCTURAS}
n_total = 0
for nombre, d in cargar(COLS):
    n_total += len(d[BASE])
    for e in ESTRUCTURAS:
        suma[e] += d[e][COLS].sum()
    if VERBOSO:  # tiempo promedio por escenario de cada estructura, mapa por mapa
        t = {e: d[e]["execution_time"].mean() for e in ESTRUCTURAS}
        print(f"{nombre:35s} n={len(d[BASE]):6d}  " + "  ".join(f"{e}={t[e]:.0f}us" for e in ESTRUCTURAS))

if n_total == 0:
    sys.exit("Sin datos para este grupo.")

media = {e: suma[e] / n_total for e in ESTRUCTURAS}

print(f"\nGrupo: {GRUPO} | escenarios comparados: {n_total:,}\n")
print("Promedios por escenario")
print(f"{'estructura':<20} {'close_bytes':>14} {'tiempo (us)':>14}")
for e in ESTRUCTURAS:
    print(f"{e:<20} {media[e]['close_bytes']:>14,.0f} {media[e]['execution_time']:>14,.1f}")


def matriz(titulo, col, formato):
    print(f"\n{titulo}")
    print("(fila frente a columna: negativo = la fila es mejor)")
    print(f"{'':<20}" + "".join(f"{r:>20}" for r in ESTRUCTURAS))
    for e in ESTRUCTURAS:
        celdas = ["—" if e == r else formato(media[e][col], media[r][col]) for r in ESTRUCTURAS]
        print(f"{e:<20}" + "".join(f"{c:>20}" for c in celdas))


matriz("TIEMPO, diferencia relativa", "execution_time", lambda a, b: f"{(a / b - 1) * 100:+.1f}%")
matriz("TIEMPO, diferencia en microsegundos", "execution_time", lambda a, b: f"{a - b:+,.0f} us")
matriz("MEMORIA (close_bytes), diferencia relativa", "close_bytes", lambda a, b: f"{(a / b - 1) * 100:+.1f}%")