import numpy as np
import pandas as pd
from pathlib import Path

path_to_go = "test_result/"

test_results = Path(f"{path_to_go}")

test_files = test_results.iterdir()

for file in test_files:
    data = pd.read_csv(file, sep="\t")
    expected = data["expected_distance"]
    actual = data["actual_distance"]

    # tolerancia relativa del 1% respecto a la distancia esperada
    matches = np.isclose(actual, expected, rtol=0.0001)
    if not matches.all():
        print(f"Discrepancia en {file.name}")
        break
    print(f"Todo bien en el dataframe {file.name}")
