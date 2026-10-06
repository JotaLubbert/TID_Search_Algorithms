import numpy as np
import pandas as pd
from pathlib import Path
import matplotlib
matplotlib.use("Agg")  # solo guarda imágenes, no abre ventanas
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter

TSV_DIR = Path("test_result")
OUT_DIR = Path("generated_output/comparacion")
CACHE = OUT_DIR / "datos.pkl"
BASELINE = "binary-heap"   # las demás estructuras se comparan contra esta
MAPS_TO_PLOT = ["arena2", "den312d", "hrt201n", "orz900d"]
N_BINS = 20                # tramos de largo de camino en los gráficos por mapa
MIN_PER_BIN = 5            # un tramo con menos escenarios no se dibuja
OPTIMALITY_TOL = 1e-6      # el .scen trae la distancia esperada con 8 decimales

# path se deja fuera a propósito: es la columna más pesada y no se usa
COLUMNS = ["start_x", "start_y", "goal_x", "goal_y", "expected_distance", "actual_distance",
           "path_len", "expansions", "generated", "open_bytes", "close_bytes", "execution_time"]

# el color sigue a la estructura, no a su posición: agregar o quitar una carpeta no repinta a las otras
STRUCTURE_STYLE = {
    "binary-heap": ("BinaryHeap", "#2a78d6"),
    "radix-heap": ("RadixHeap", "#eb6834"),
    "veb-tree": ("vEB", "#1baf7a"),
}
SPARE_COLORS = ["#eda100", "#e87ba4"]  # siguientes colores de la paleta, para carpetas nuevas

INK = {"primary": "#0b0b0b", "secondary": "#52514e", "muted": "#898781",
       "grid": "#e1e0d9", "axis": "#c3c2b7", "surface": "#fcfcfb"}

plt.rcParams.update({
    "figure.facecolor": INK["surface"], "axes.facecolor": INK["surface"], "savefig.facecolor": INK["surface"],
    "font.family": "sans-serif", "font.size": 10,
    "axes.edgecolor": INK["axis"], "axes.linewidth": 0.8,
    "axes.spines.top": False, "axes.spines.right": False,
    "axes.titlesize": 11, "axes.titlelocation": "left", "axes.titlecolor": INK["primary"], "axes.titlepad": 10,
    "axes.labelcolor": INK["secondary"],
    "axes.grid": True, "axes.grid.axis": "y", "axes.axisbelow": True,
    "grid.color": INK["grid"], "grid.linewidth": 0.6, "grid.linestyle": "-",
    "xtick.color": INK["muted"], "ytick.color": INK["muted"],
    "xtick.labelcolor": INK["secondary"], "ytick.labelcolor": INK["secondary"],
    "xtick.major.size": 0, "ytick.major.size": 0,
    "lines.linewidth": 1.6, "lines.solid_capstyle": "round", "lines.solid_joinstyle": "round",
    "legend.frameon": False,
})


def fmt_number(value, decimals=0):
    #formato chileno: punto para miles y coma para decimales
    text = f"{value:,.{decimals}f}"
    return text.replace(",", "_").replace(".", ",").replace("_", ".")


def apply_ratio_format(ax, axis="y"):
    #mismos decimales en todas las marcas del eje, los justos para que no se repitan.
    #Se llama después de tight_layout porque el ajuste del tamaño puede cambiar las marcas.
    target = ax.yaxis if axis == "y" else ax.xaxis
    ticks = target.get_majorticklocs()
    step = np.min(np.diff(ticks)) if len(ticks) > 1 else 1
    #los decimales justos para escribir el paso exacto (0,025 necesita 3, no 2)
    decimals = next(d for d in range(7) if abs(round(step * 10**d) - step * 10**d) < 1e-6)
    target.set_major_formatter(FuncFormatter(lambda v, _: fmt_number(v, decimals) + "×"))


def keep_reference_visible(ax):
    #la línea 1× siempre dentro del eje y con aire a ambos lados, aunque los datos queden lejos de ella
    y_min, y_max = ax.get_ylim()
    span = max(y_max, 1) - min(y_min, 1)
    ax.set_ylim(min(y_min, 1 - 0.06 * span), max(y_max, 1 + 0.06 * span))


def panel_legend(ax, loc):
    #fondo del color del gráfico y sin borde: si la leyenda cae sobre una curva, el texto se sigue leyendo
    ax.legend(loc=loc, fontsize=8.5, labelcolor=INK["secondary"], handlelength=1.6, borderaxespad=0.3,
              frameon=True, facecolor=INK["surface"], edgecolor="none", framealpha=0.92)


def structure_styles(structures):
    styles = {}
    spare = iter(SPARE_COLORS)
    for name in structures:
        if name in STRUCTURE_STYLE:
            styles[name] = STRUCTURE_STYLE[name]
        else:
            color = next(spare, None)
            if color is None:
                raise ValueError("hay más estructuras que colores distinguibles; separa los gráficos en grupos")
            styles[name] = (name, color)
    #la referencia va primero para que la leyenda siempre parta igual
    return dict(sorted(styles.items(), key=lambda item: item[0] != BASELINE))


def read_tsvs(structures):
    frames = []
    for structure in structures:
        for tsv in sorted((TSV_DIR / structure).glob("*.tsv")):
            df = pd.read_csv(tsv, sep="\t", usecols=COLUMNS)
            df["map"] = tsv.name.removesuffix(".map.scen.tsv")
            df["row"] = np.arange(len(df))
            df["structure"] = structure
            frames.append(df)
    return pd.concat(frames, ignore_index=True)


def load_results():
    structures = sorted(p.name for p in TSV_DIR.iterdir() if p.is_dir())
    if BASELINE not in structures:
        raise ValueError(f"falta la carpeta de referencia {TSV_DIR / BASELINE}")
    newest_tsv = max(t.stat().st_mtime for t in TSV_DIR.glob("*/*.tsv"))
    #leer los TSV completos tarda; si nada cambió desde la última vez se reutiliza el caché
    if CACHE.exists() and CACHE.stat().st_mtime > newest_tsv:
        data = pd.read_pickle(CACHE)
        if sorted(data["structure"].unique()) == structures:
            return data

    data = read_tsvs(structures)
    #cada fila se empareja con la misma fila de la referencia: mismo mapa, mismo escenario
    base_cols = ["start_x", "start_y", "goal_x", "goal_y", "actual_distance",
                 "execution_time", "open_bytes", "expansions"]
    base = data[data["structure"] == BASELINE].set_index(["map", "row"])[base_cols].add_suffix("_base")
    data = data.join(base, on=["map", "row"])
    coords = ["start_x", "start_y", "goal_x", "goal_y"]
    same_scenario = (data[coords].to_numpy() == data[[c + "_base" for c in coords]].to_numpy()).all(axis=1)
    if not same_scenario.all():
        bad = data.loc[~same_scenario, ["structure", "map", "row"]].head()
        raise ValueError(f"hay filas que no corresponden al mismo escenario en todas las estructuras:\n{bad}")

    data["time_ms"] = data["execution_time"] / 1000
    data["open_kb"] = data["open_bytes"] / 1024
    data["us_per_expansion"] = data["execution_time"] / data["expansions"]
    #un tiempo de 0 µs (problemas triviales) no sirve como divisor
    data["time_ratio"] = data["execution_time"] / data["execution_time_base"].replace(0, np.nan)
    data["memory_ratio"] = data["open_bytes"] / data["open_bytes_base"]
    data["expansions_ratio"] = data["expansions"] / data["expansions_base"]
    data["error"] = (data["actual_distance"] - data["expected_distance"]).abs()
    data["optimal"] = data["error"] <= OPTIMALITY_TOL
    data["same_as_base"] = (data["actual_distance"] - data["actual_distance_base"]).abs() <= OPTIMALITY_TOL

    CACHE.parent.mkdir(parents=True, exist_ok=True)
    data.to_pickle(CACHE)
    return data


def print_optimality(data, styles):
    g = data.groupby("structure")
    table = pd.DataFrame({
        "escenarios": g.size(),
        "óptimos (%)": g["optimal"].mean() * 100,
        "error máx": g["error"].max(),
        f"igual a {styles[BASELINE][0]} (%)": g["same_as_base"].mean() * 100,
    }).rename(index=lambda s: styles[s][0])
    print(f"\nOptimalidad (tolerancia {OPTIMALITY_TOL:g} contra la distancia del benchmark)")
    print(table.to_string(float_format=lambda v: f"{v:.4g}"))


def save_summaries(data, styles):
    #las tablas son la versión exacta de los gráficos
    metrics = {
        "tiempo_ms_mediana": ("time_ms", "median"),
        "tiempo_ms_total": ("time_ms", "sum"),
        "open_kb_mediana": ("open_kb", "median"),
        "expansiones_mediana": ("expansions", "median"),
        "us_por_expansion_mediana": ("us_per_expansion", "median"),
        "tiempo_vs_base_mediana": ("time_ratio", "median"),
        "memoria_vs_base_mediana": ("memory_ratio", "median"),
        "expansiones_vs_base_mediana": ("expansions_ratio", "median"),
        "optimos_pct": ("optimal", "mean"),
    }
    per_map = data.groupby(["map", "structure"]).agg(**metrics)
    per_map["optimos_pct"] *= 100
    per_map.to_csv(OUT_DIR / "resumen_por_mapa.csv")

    overall = data.groupby("structure").agg(**metrics).rename(index=lambda s: styles[s][0])
    overall["optimos_pct"] *= 100
    overall.to_csv(OUT_DIR / "resumen_global.csv")
    print("\nResumen de todos los mapas (medianas por escenario)")
    cols = ["tiempo_ms_total", "us_por_expansion_mediana", "tiempo_vs_base_mediana",
            "memoria_vs_base_mediana", "expansiones_vs_base_mediana"]
    print(overall[cols].to_string(float_format=lambda v: f"{v:.4g}"))


def add_end_labels(ax, ends):
    #etiqueta cada línea en su extremo derecho, salvo que choquen: ahí basta la leyenda.
    #Devuelve True si las líneas terminan demasiado juntas para etiquetarlas.
    if len(ends) < 2:
        return False
    y_min, y_max = ax.get_ylim()
    positions = sorted((y - y_min) / (y_max - y_min) for _, y, _ in ends)
    if min(b - a for a, b in zip(positions, positions[1:])) < 0.07:
        return True
    for x, y, label in ends:
        ax.annotate(label, xy=(x, y), xytext=(6, 0), textcoords="offset points",
                    va="center", fontsize=9, color=INK["secondary"])
    return False


def plot_map(data, map_name, styles):
    df = data[data["map"] == map_name]
    if df.empty:
        print(f"aviso: no hay resultados para el mapa {map_name}")
        return
    base_label = styles[BASELINE][0]
    edges = np.linspace(df["expected_distance"].min(), df["expected_distance"].max(), N_BINS + 1)
    centers = (edges[:-1] + edges[1:]) / 2
    df = df.assign(bin=pd.cut(df["expected_distance"], edges, labels=False, include_lowest=True))

    fig, axes = plt.subplots(2, 3, figsize=(15, 9.6), sharex=True)
    #arriba los valores absolutos, abajo cada escenario dividido por el mismo escenario en la referencia
    panels = [
        (axes[0, 0], "time_ms", "Tiempo de ejecución", "Milisegundos", False),
        (axes[0, 1], "open_kb", "Memoria de la open", "Kilobytes", False),
        (axes[0, 2], "expansions", "Nodos expandidos", "Nodos", False),
        (axes[1, 0], "time_ratio", f"Tiempo relativo a {base_label}", f"Veces el tiempo de {base_label}", True),
        (axes[1, 1], "memory_ratio", f"Memoria relativa a {base_label}", f"Veces la memoria de {base_label}", True),
        (axes[1, 2], "expansions_ratio", f"Expansiones relativas a {base_label}", f"Veces las expansiones de {base_label}", True),
    ]
    for ax, column, title, y_label, relative in panels:
        ends = []
        medians = {}
        if relative:
            #la referencia es la línea 1× en el color de la referencia: sobre ella la estructura
            #es peor, bajo ella mejor. Va primero para encabezar la leyenda.
            ax.axhline(1, color=styles[BASELINE][1], linewidth=1.2, zorder=1,
                       label=f"{base_label} (referencia = 1×)")
        for structure, (label, color) in styles.items():
            if relative and structure == BASELINE:
                continue
            stats = df[df["structure"] == structure].groupby("bin")[column].agg(["median", "size"])
            stats = stats[stats["size"] >= MIN_PER_BIN]
            x = centers[stats.index.astype(int)]
            ax.plot(x, stats["median"], color=color, label=label)
            ends.append((x[-1], stats["median"].iloc[-1], label))
            medians[structure] = stats["median"]
        ax.set_title(title)
        ax.set_ylabel(y_label)
        #sharex esconde los números del eje x en la fila de arriba; aquí se vuelven a mostrar
        ax.tick_params(labelbottom=True)
        ax.set_xlabel("Largo del camino óptimo")
        ax.xaxis.set_major_formatter(FuncFormatter(lambda v, _: fmt_number(v)))
        if relative:
            keep_reference_visible(ax)
        else:
            ax.set_ylim(bottom=0)
            ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: fmt_number(v, 0 if v >= 10 or v == 0 else 1)))
        x_left = ax.get_xlim()[0]
        ax.set_xlim(x_left, edges[-1] + (edges[-1] - edges[0]) * 0.13)
        #las curvas absolutas crecen hacia la derecha, así que arriba a la izquierda queda libre
        panel_legend(ax, "best" if relative else "upper left")
        add_end_labels(ax, ends)
        #curvas que nunca se separan más de un 2% del alto del eje: en pantalla una tapa a la otra
        y_min, y_max = ax.get_ylim()
        overlapping = not relative and len(medians) > 1 and all(
            ((m - medians[BASELINE]).abs() / (y_max - y_min)).max() < 0.02
            for s, m in medians.items() if s != BASELINE)
        if overlapping:
            #se avisa para que no parezca que falta una curva; va abajo a la derecha porque
            #las curvas crecen con el largo del camino y esa esquina queda libre
            ax.text(0.98, 0.04, "Las curvas casi coinciden;\nla diferencia está en el panel de abajo",
                    transform=ax.transAxes, ha="right", va="bottom", fontsize=8.5, color=INK["muted"])
    n_scen = df[df["structure"] == BASELINE].shape[0]
    fig.suptitle(f"{map_name}: estructuras de la open en los mismos caminos", x=0.012, ha="left",
                 fontsize=14, color=INK["primary"], y=0.995)
    fig.text(0.012, 0.957, f"Mediana por tramo de largo de camino · {fmt_number(n_scen)} escenarios · "
             f"abajo, cada camino dividido por el mismo camino resuelto con {base_label}",
             fontsize=9.5, color=INK["secondary"])
    fig.tight_layout(rect=(0, 0, 1, 0.945), h_pad=2.6, w_pad=2.5)
    for ax in axes[1]:
        apply_ratio_format(ax)
    out = OUT_DIR / f"mapa_{map_name}.png"
    fig.savefig(out, dpi=160)
    plt.close(fig)
    print(f"guardado {out}")


def plot_global(data, styles):
    #distribución acumulada: para cada x, qué fracción de los caminos queda en x veces la referencia o menos
    base_label = styles[BASELINE][0]
    others = {s: v for s, v in styles.items() if s != BASELINE}
    if not others:
        return
    fig, axes = plt.subplots(1, 3, figsize=(15, 5.9))
    panels = [
        (axes[0], "time_ratio", "Tiempo", "el tiempo"),
        (axes[1], "memory_ratio", "Memoria de la open", "la memoria"),
        (axes[2], "expansions_ratio", "Nodos expandidos", "las expansiones"),
    ]
    for ax, column, title, what in panels:
        values_all = data.loc[data["structure"] != BASELINE, column].dropna()
        lo, hi = values_all.quantile(0.005), values_all.quantile(0.995)
        #la referencia como línea vertical en su color: cada camino suyo dividido por sí mismo da 1×
        ax.axvline(1, color=styles[BASELINE][1], linewidth=1.2, zorder=1,
                   label=f"{base_label} (referencia = 1×)")
        for structure, (label, color) in others.items():
            values = data.loc[data["structure"] == structure, column].dropna().to_numpy()
            #2.001 cuantiles bastan para una curva suave y evitan dibujar 155 mil puntos
            fraction = np.linspace(0, 1, 2001)
            median = np.median(values)
            higher = (values > 1).mean() * 100
            #la leyenda trae la lectura principal: la mediana y en cuántos caminos queda sobre la referencia
            median_text = fmt_number(median, 3 if abs(median - 1) < 0.01 else 2)
            higher_text = fmt_number(higher, 0 if higher >= 10 else 1)
            ax.plot(np.quantile(values, fraction), fraction, color=color,
                    label=f"{label}: mediana {median_text}×\n{higher_text}% de los caminos sobre 1×")
        ax.set_xlim(min(lo, 0.95), max(hi, 1.05))
        ax.set_ylim(0, 1)
        ax.set_title(title)
        ax.set_xlabel(f"Veces {what} de {base_label} en el mismo camino")
        ax.grid(axis="x")
        ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:.0%}"))
        #cada panel tiene su zona libre en un lugar distinto; "best" elige la que menos tapa
        panel_legend(ax, "best")
    axes[0].set_ylabel("Porcentaje de caminos con ese valor o menos")
    n_scen = (data["structure"] == BASELINE).sum()
    fig.suptitle(f"Todos los mapas: cada estructura comparada con {base_label} en los mismos caminos",
                 x=0.012, ha="left", fontsize=14, color=INK["primary"], y=0.985)
    fig.text(0.012, 0.865,
             f"{fmt_number(n_scen)} caminos de {data['map'].nunique()} mapas. Cada camino de una estructura se divide por "
             f"ese mismo camino en {base_label}, que por eso queda como la línea vertical en 1×.\n"
             "Cómo leer una curva: un punto (x, y) dice que en el y% de los caminos la estructura tuvo x veces "
             f"el valor de {base_label} o menos. A la izquierda de 1× gana; a la derecha pierde.",
             fontsize=9.5, color=INK["secondary"], linespacing=1.5)
    fig.tight_layout(rect=(0, 0, 1, 0.85), w_pad=2.5)
    for ax in axes:
        apply_ratio_format(ax, "x")
    out = OUT_DIR / "todos_los_mapas.png"
    fig.savefig(out, dpi=160)
    plt.close(fig)
    print(f"guardado {out}")


if __name__ == "__main__":
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    data = load_results()
    styles = structure_styles(sorted(data["structure"].unique()))
    print_optimality(data, styles)
    save_summaries(data, styles)
    for map_name in MAPS_TO_PLOT:
        plot_map(data, map_name, styles)
    plot_global(data, styles)
