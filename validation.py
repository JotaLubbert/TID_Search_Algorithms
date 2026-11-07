from comun import *
import numpy as np

# Cuenta, por estructura, en cuántos escenarios la distancia obtenida coincide con la del .scen
tot = {e: [0, 0, 0] for e in ESTRUCTURAS}  # escenarios, coinciden (atol 1e-6), coinciden (rtol 1e-4)
ejemplos = []
for nombre, d in cargar(["expected_distance", "actual_distance"]):
    for e in ESTRUCTURAS:
        esperado = d[e]["expected_distance"].to_numpy()
        obtenido = d[e]["actual_distance"].to_numpy()
        bien = np.isclose(obtenido, esperado, rtol=0, atol=1e-6)
        tot[e][0] += len(esperado)
        tot[e][1] += int(bien.sum())
        tot[e][2] += int(np.isclose(obtenido, esperado, rtol=1e-4, atol=0).sum())
        if e == BASE:
            ejemplos += [(nombre, int(i), esperado[i], obtenido[i]) for i in np.flatnonzero(~bien)[:2]]

print(f"Grupo: {GRUPO}")
for e in ESTRUCTURAS:
    t = tot[e]
    print(f"{e:<20} escenarios {t[0]:,}  coinciden (|dif| ≤ 1e-6): {t[1]:,}  (rtol 1e-4: {t[2]:,})")
for m in ejemplos[:5]:
    print("    ejemplo de diferencia:", m)