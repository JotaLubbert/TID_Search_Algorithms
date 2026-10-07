from pathlib import Path

#Borra los resultados de la corrida anterior antes de volver a correr las pruebas.
#create_stat_file agrega filas al final si el archivo ya existe, así que sin esto
#los resultados nuevos se mezclarían con los viejos.
#La ruta se arma desde la ubicación del script, así funciona sin importar desde dónde se corra.
test_res = Path(__file__).parent / "test_result"

if not test_res.is_dir():
    print(f"No existe la carpeta {test_res}, no hay nada que borrar")
else:
    data_struct = sorted(folder for folder in test_res.iterdir() if folder.is_dir())
    for folder in data_struct:
        #solo los .tsv que escribe el programa; la carpeta se mantiene porque create_stat_file la necesita
        files = list(folder.glob("*.tsv"))
        for file in files:
            file.unlink()
        print(f"{folder.name}: {len(files)} archivos eliminados")
