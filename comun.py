import sys
from pathlib import Path

import pandas as pd

# Estructuras comparadas (nombres de las carpetas en test_result/)
ESTRUCTURAS = ["hashmap", "hashmap_hash_robin", "robin_hood", "judy"]
BASE = "hashmap"  # referencia principal

# Uso: python script.py [grupo] [-v]    grupo = tradicionales (por defecto) o laberintos
_args = [a for a in sys.argv[1:] if not a.startswith("-")]
GRUPO = _args[0] if _args else "tradicionales"
VERBOSO = "-v" in sys.argv
RAIZ = Path("test_result")


def carpeta(e):
    return RAIZ / e / GRUPO


def cargar(columnas):
    """Por cada archivo presente en TODAS las estructuras devuelve (nombre, {estructura: DataFrame}),
    con las mismas n primeras filas para todas (sirve aunque alguna corrida esté incompleta)."""
    base = carpeta(BASE)
    if not base.exists():
        print(f"No existe {base}")
        return
    for ruta_base in sorted(base.iterdir()):
        nombre = ruta_base.name
        datos = {}
        for e in ESTRUCTURAS:
            ruta = carpeta(e) / nombre
            if ruta.exists():
                datos[e] = pd.read_csv(ruta, sep="\t", usecols=columnas)
        if len(datos) < len(ESTRUCTURAS):
            continue
        n = min(len(d) for d in datos.values())
        yield nombre, {e: d.iloc[:n].reset_index(drop=True) for e, d in datos.items()}