import numpy as np
import pandas as pd
from pathlib import Path

path_to_go = "test_result/"

test_results = Path(f"{path_to_go}")

#test_result tiene una carpeta por estructura, y dentro de cada una los .tsv de cada mapa
structures = sorted(folder for folder in test_results.iterdir() if folder.is_dir())

for structure in structures:
    bad_files = 0
    test_files = sorted(structure.glob("*.tsv"))
    for file in test_files:
        #solo las dos columnas que se comparan; la del camino es la más pesada y no se usa
        data = pd.read_csv(file, sep="\t", usecols=["expected_distance", "actual_distance"])
        expected = data["expected_distance"]
        actual = data["actual_distance"]

        # tolerancia relativa del 0,01% respecto a la distancia esperada
        matches = np.isclose(actual, expected, rtol=0.0001)
        if not matches.all():
            print(f"Discrepancia en {structure.name}/{file.name}: {(~matches).sum()} filas")
            bad_files += 1
    if bad_files == 0:
        print(f"Todo bien en {structure.name}: {len(test_files)} archivos")
    else:
        print(f"{structure.name}: {bad_files} de {len(test_files)} archivos con discrepancias")
