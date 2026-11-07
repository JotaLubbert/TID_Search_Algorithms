from comun import *

tot = {e: 0 for e in ESTRUCTURAS}  # microsegundos
escenarios = 0
for nombre, d in cargar(["execution_time"]):
    escenarios += len(d[BASE])
    for e in ESTRUCTURAS:
        tot[e] += int(d[e]["execution_time"].sum())

if escenarios == 0:
    sys.exit("Sin datos para este grupo.")

print(f"Grupo: {GRUPO} | escenarios sumados: {escenarios:,}\n")
print(f"{'estructura':<20} {'tiempo total':>14} {'vs hashmap':>12} {'vs hashmap_hash_robin':>22}")
for e in ESTRUCTURAS:
    seg = tot[e] / 1e6
    p0 = "base" if e == "hashmap" else f"{(tot[e] / tot['hashmap'] - 1) * 100:+.1f}%"
    p1 = "base" if e == "hashmap_hash_robin" else f"{(tot[e] / tot['hashmap_hash_robin'] - 1) * 100:+.1f}%"
    print(f"{e:<20} {seg:>12,.2f} s {p0:>12} {p1:>22}")