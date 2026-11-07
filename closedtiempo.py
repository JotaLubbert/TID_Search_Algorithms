from comun import *

COLS = ["execution_time", "closed_get_ns", "closed_insert_ns", "closed_gets", "closed_inserts"]
tot = {e: dict.fromkeys(COLS, 0) for e in ESTRUCTURAS}
escenarios = 0
for nombre, d in cargar(COLS):
    escenarios += len(d[BASE])
    for e in ESTRUCTURAS:
        for c in COLS:
            tot[e][c] += int(d[e][c].sum())

if escenarios == 0:
    sys.exit("Sin datos para este grupo.")

cerr = {e: (tot[e]["closed_get_ns"] + tot[e]["closed_insert_ns"]) / 1e9 for e in ESTRUCTURAS}


def rel(e, r):
    return "base" if e == r else f"{(cerr[e] / cerr[r] - 1) * 100:+.1f}%"


print(f"Grupo: {GRUPO} | escenarios sumados: {escenarios:,}\n")
print(f"{'estructura':<20} {'get (s)':>9} {'insert (s)':>11} {'Closed (s)':>11} {'ns/get':>8} {'ns/insert':>10}"
      f" {'% de A*':>8} {'vs hashmap':>11} {'vs hash_robin':>14}")
for e in ESTRUCTURAS:
    t = tot[e]
    get_s = t["closed_get_ns"] / 1e9
    ins_s = t["closed_insert_ns"] / 1e9
    ns_get = t["closed_get_ns"] / t["closed_gets"]
    ns_ins = t["closed_insert_ns"] / t["closed_inserts"]
    share = cerr[e] / (t["execution_time"] / 1e6) * 100  # execution_time está en microsegundos
    print(f"{e:<20} {get_s:>9.1f} {ins_s:>11.1f} {cerr[e]:>11.1f} {ns_get:>8.1f} {ns_ins:>10.1f}"
          f" {share:>7.1f}% {rel(e, 'hashmap'):>11} {rel(e, 'hashmap_hash_robin'):>14}")