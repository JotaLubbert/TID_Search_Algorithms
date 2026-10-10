import numpy as np
import pandas as pd
from pathlib import Path
import matplotlib
matplotlib.use("Agg")  # solo guarda imágenes, no abre ventanas
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter
from matplotlib.transforms import offset_copy

TSV_DIR = Path("test_result")
OUT_DIR = Path("generated_output/comparacion")
CACHE = OUT_DIR / "datos.pkl"
BASELINE = "binary-heap"   # las demás estructuras se comparan contra esta
MAPS_TO_PLOT = ["arena2", "den312d", "hrt201n", "orz900d"]
N_BINS = 20                # tramos de largo de camino en los gráficos por mapa
MIN_PER_BIN = 5            # un tramo con menos escenarios no se dibuja
OPTIMALITY_TOL = 1e-6      # el .scen trae la distancia esperada con 8 decimales
#el tiempo se guarda en µs enteros: en caminos de pocos µs dos estructuras empatan o difieren
#solo por el redondeo, así que el tiempo relativo se calcula únicamente desde este umbral
MIN_US_FOR_RATIO = 50
#para el desglose por tamaño de la open: BinaryHeap reserva 48 bytes por nodo (Reverse<SearchNode>)
#más los 24 del Vec, así que de open_bytes sale su capacidad en nodos (una potencia de 2)
BASELINE_NODE_BYTES = 48
BASELINE_STRUCT_BYTES = 24
OPEN_SMALLEST_BIN = 128    # las capacidades menores se juntan en un solo tramo
MIN_PER_OPEN_BIN = 50      # un tamaño con menos caminos no se dibuja
#niveles de dificultad de una ruta; los cortes no son fijos, se calculan con los datos (with_difficulty)
DIFFICULTY_LABELS = ["Fácil", "Medio", "Difícil"]

#métricas de las tablas resumen: nombre de la columna -> (columna de los datos, agregación)
SUMMARY_METRICS = {
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

# path se deja fuera a propósito: es la columna más pesada y no se usa
COLUMNS = ["start_x", "start_y", "goal_x", "goal_y", "expected_distance", "actual_distance",
           "path_len", "expansions", "generated", "open_bytes", "close_bytes", "execution_time"]

# el color sigue a la estructura, no a su posición: agregar o quitar una carpeta no repinta a las otras
STRUCTURE_STYLE = {
    "binary-heap": ("BinaryHeap", "#2a78d6"),
    "radix-heap": ("RadixHeap", "#eb6834"),
    "veb-tree": ("vEB", "#1baf7a"),
    #violeta y no el siguiente de la paleta (amarillo): el amarillo se confunde con el naranja del
    #radix original, que es justo con el que más se compara
    "radix-alt": ("RadixAlt", "#4a3aa7"),
    #con cinco series ningún color pasa contra todos los pares; el amarillo pasa entre vecinos y su
    #único choque es leve (con el naranja, 13,7 de separación contra un mínimo de 15). La leyenda y
    #las etiquetas al final de cada línea evitan que la identidad dependa solo del color
    "radix-alt-round": ("RadixAltRound", "#eda100"),
}
SPARE_COLORS = ["#e87ba4"]  # siguiente color de la paleta, para una carpeta nueva

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


def fmt_value(value):
    #tres cifras significativas: 0,123 · 1,23 · 12,3 · 123
    decimals = 0 if value >= 100 else 1 if value >= 10 else 2 if value >= 1 else 3
    return fmt_number(value, decimals)


def apply_tick_format(ax, axis="y", suffix=""):
    #mismos decimales en todas las marcas del eje, los justos para que no se repitan.
    #Se llama después de tight_layout porque el ajuste del tamaño puede cambiar las marcas.
    target = ax.yaxis if axis == "y" else ax.xaxis
    ticks = target.get_majorticklocs()
    step = np.min(np.diff(ticks)) if len(ticks) > 1 else 1
    #los decimales justos para escribir el paso exacto (0,025 necesita 3, no 2)
    decimals = next(d for d in range(7) if abs(round(step * 10**d) - step * 10**d) < 1e-6)
    target.set_major_formatter(FuncFormatter(lambda v, _: fmt_number(v, decimals) + suffix))


def apply_ratio_format(ax, axis="y"):
    apply_tick_format(ax, axis, "×")


def keep_reference_visible(ax):
    #la línea 1× siempre dentro del eje y con aire a ambos lados, aunque los datos queden lejos de ella
    y_min, y_max = ax.get_ylim()
    span = max(y_max, 1) - min(y_min, 1)
    ax.set_ylim(min(y_min, 1 - 0.06 * span), max(y_max, 1 + 0.06 * span))


def plot_series(ax, drawn, name, x, y, color, label, twin_label):
    #si otra estructura ya dibujó exactamente la misma curva, la nueva la taparía entera:
    #la de abajo se engrosa para que asome como un borde alrededor de la nueva, y la leyenda lo dice
    x, y = np.asarray(x, dtype=float), np.asarray(y, dtype=float)
    for other, (other_x, other_y, other_line) in drawn.items():
        if np.array_equal(other_x, x) and np.array_equal(other_y, y):
            other_line.set_linewidth(4.5)
            other_line.set_zorder(2)
            ax.plot(x, y, color=color, label=twin_label(other), zorder=3)
            return
    line, = ax.plot(x, y, color=color, label=label)
    drawn[name] = (x, y, line)


def panel_legend(ax, ncol=2):
    #la leyenda va bajo el panel y no adentro: las curvas cambian con los datos y, con cualquier
    #ubicación fija o "best", en algún mapa terminaba tapando una curva.
    #Se ancla a una distancia fija en pulgadas bajo el eje (no en fracción del panel), así queda
    #debajo de los números y del título del eje x sin importar el alto del panel
    below_axis = offset_copy(ax.transAxes, fig=ax.figure, y=-0.58, units="inches")
    ax.legend(loc="upper center", bbox_to_anchor=(0.5, 0), bbox_transform=below_axis, ncol=ncol,
              fontsize=8.5, labelcolor=INK["secondary"], handlelength=1.6, columnspacing=1.4)


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
    #una carpeta todavía vacía (estructura sin correr) no cuenta
    structures = sorted(p.name for p in TSV_DIR.iterdir() if p.is_dir() and any(p.glob("*.tsv")))
    if BASELINE not in structures:
        raise ValueError(f"falta la carpeta de referencia {TSV_DIR / BASELINE}")
    newest_tsv = max(t.stat().st_mtime for t in TSV_DIR.glob("*/*.tsv"))
    #leer los TSV completos tarda; si nada cambió desde la última vez se reutiliza el caché
    if CACHE.exists() and CACHE.stat().st_mtime > newest_tsv:
        data = pd.read_pickle(CACHE)
        if sorted(data["structure"].unique()) == structures:
            return with_time_ratio(data)

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
    data["memory_ratio"] = data["open_bytes"] / data["open_bytes_base"]
    data["expansions_ratio"] = data["expansions"] / data["expansions_base"]
    data["error"] = (data["actual_distance"] - data["expected_distance"]).abs()
    data["optimal"] = data["error"] <= OPTIMALITY_TOL
    data["same_as_base"] = (data["actual_distance"] - data["actual_distance_base"]).abs() <= OPTIMALITY_TOL

    CACHE.parent.mkdir(parents=True, exist_ok=True)
    data.to_pickle(CACHE)
    return with_time_ratio(data)


def with_time_ratio(data):
    #va fuera del caché para que cambiar el umbral no obligue a releer los TSV
    base = data["execution_time_base"]
    data["time_ratio"] = data["execution_time"] / base.where(base >= MIN_US_FOR_RATIO)
    return data


def with_difficulty(data):
    #la dificultad sale del bucket de MovingAI, la 1ra columna del .scen: el largo óptimo dividido por 4
    #y redondeado hacia abajo. El .tsv no lo trae, pero sí el largo óptimo (expected_distance).
    #Los cortes son absolutos (los mismos para todos los mapas) y se calculan cada vez: son los terciles
    #del bucket entre todos los escenarios, contando una vez cada escenario (la fila de la referencia).
    #Así cada nivel queda con ~1/3 de las rutas y, si se agregan mapas, los cortes se ajustan solos.
    #Va fuera del caché, igual que el tiempo relativo.
    data["bucket"] = (data["expected_distance"] // 4).astype(int)
    cuts = data.loc[data["structure"] == BASELINE, "bucket"].quantile([1 / 3, 2 / 3]).to_numpy()
    #right=False: un bucket igual al corte pasa al nivel de arriba
    data["difficulty"] = pd.cut(data["bucket"], [-np.inf, *cuts, np.inf], right=False, labels=DIFFICULTY_LABELS)
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
    per_map = data.groupby(["map", "structure"]).agg(**SUMMARY_METRICS)
    per_map["optimos_pct"] *= 100
    per_map.to_csv(OUT_DIR / "resumen_por_mapa.csv")

    overall = data.groupby("structure").agg(**SUMMARY_METRICS).rename(index=lambda s: styles[s][0])
    overall["optimos_pct"] *= 100
    overall.to_csv(OUT_DIR / "resumen_global.csv")
    print("\nResumen de todos los mapas (medianas por escenario)")
    cols = ["tiempo_ms_total", "us_por_expansion_mediana", "tiempo_vs_base_mediana",
            "memoria_vs_base_mediana", "expansiones_vs_base_mediana"]
    print(overall[cols].to_string(float_format=lambda v: f"{v:.4g}"))


def save_difficulty_summary(data, styles):
    #qué rutas quedaron en cada nivel, contando una vez cada escenario (la fila de la referencia)
    base = data[data["structure"] == BASELINE]
    levels = base.groupby("difficulty", observed=True).agg(
        caminos=("bucket", "size"), mapas=("map", "nunique"),
        bucket_min=("bucket", "min"), bucket_max=("bucket", "max"),
        largo_min=("expected_distance", "min"), largo_max=("expected_distance", "max"))
    #count (y no size) dice cuántos caminos entran al tiempo relativo después del umbral de µs
    table = data.groupby(["difficulty", "structure"], observed=True).agg(
        **SUMMARY_METRICS, caminos_con_tiempo_relativo=("time_ratio", "count"))
    table["optimos_pct"] *= 100
    table.rename(index=lambda s: styles[s][0] if s in styles else s, level="structure") \
        .to_csv(OUT_DIR / "resumen_por_dificultad.csv")

    print("\nNiveles de dificultad (terciles del bucket de MovingAI, los mismos cortes para todos los mapas)")
    print(levels.to_string(float_format=lambda v: fmt_number(v, 1)))
    ratios = table["tiempo_vs_base_mediana"].unstack("structure").drop(columns=BASELINE)
    print(f"\nTiempo relativo a {styles[BASELINE][0]} por dificultad (mediana por camino)")
    print(ratios.rename(columns=lambda s: styles[s][0]).to_string(float_format=lambda v: f"{v:.4g}"))
    return levels


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

    fig, axes = plt.subplots(2, 3, figsize=(15, 11.8), sharex=True)
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
        drawn = {}
        for structure, (label, color) in styles.items():
            if relative and structure == BASELINE:
                continue
            #count (y no size) ignora los cocientes descartados por el umbral de µs
            stats = df[df["structure"] == structure].groupby("bin")[column].agg(["median", "count"])
            stats = stats[stats["count"] >= MIN_PER_BIN]
            x = centers[stats.index.astype(int)]
            plot_series(ax, drawn, label, x, stats["median"], color, label,
                        lambda other, label=label: f"{label}\n(igual a {other})")
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
        panel_legend(ax)
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
    fig, axes = plt.subplots(1, 3, figsize=(15, 7.0))
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
        drawn = {}
        for structure, (label, color) in others.items():
            values = data.loc[data["structure"] == structure, column].dropna().to_numpy()
            #2.001 cuantiles bastan para una curva suave y evitan dibujar 155 mil puntos
            fraction = np.linspace(0, 1, 2001)
            median = np.median(values)
            higher = (values > 1).mean() * 100
            #la leyenda trae la lectura principal: la mediana y en cuántos caminos queda sobre la referencia
            median_text = fmt_number(median, 3 if abs(median - 1) < 0.01 else 2)
            higher_text = fmt_number(higher, 0 if higher >= 10 else 1)
            text = f"{label}: mediana {median_text}×\n{higher_text}% de los caminos sobre 1×"
            #RadixAlt y RadixHeap expanden exactamente los mismos nodos (misma clave, mismo desempate):
            #plot_series evita que la segunda curva tape a la primera y lo dice en la leyenda
            plot_series(ax, drawn, label, np.quantile(values, fraction), fraction, color, text,
                        lambda other, text=text: f"{text}\n(igual a {other})")
        ax.set_xlim(min(lo, 0.95), max(hi, 1.05))
        ax.set_ylim(0, 1)
        ax.set_title(title)
        ax.set_xlabel(f"Veces {what} de {base_label} en el mismo camino")
        ax.grid(axis="x")
        ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:.0%}"))
        panel_legend(ax)
    axes[0].set_ylabel("Porcentaje de caminos con ese valor o menos")
    n_scen = (data["structure"] == BASELINE).sum()
    fig.suptitle(f"Todos los mapas: cada estructura comparada con {base_label} en los mismos caminos",
                 x=0.012, ha="left", fontsize=14, color=INK["primary"], y=0.985)
    fig.text(0.012, 0.895,
             f"{fmt_number(n_scen)} caminos de {data['map'].nunique()} mapas. Cada camino de una estructura se divide por "
             f"ese mismo camino en {base_label}, que por eso queda como la línea vertical en 1×.\n"
             "Cómo leer una curva: un punto (x, y) dice que en el y% de los caminos la estructura tuvo x veces "
             f"el valor de {base_label} o menos. A la izquierda de 1× gana; a la derecha pierde.",
             fontsize=9.5, color=INK["secondary"], linespacing=1.5)
    fig.tight_layout(rect=(0, 0, 1, 0.885), w_pad=2.5)
    for ax in axes:
        apply_ratio_format(ax, "x")
    out = OUT_DIR / "todos_los_mapas.png"
    fig.savefig(out, dpi=160)
    plt.close(fig)
    print(f"guardado {out}")


def plot_by_open(data, styles):
    #cada camino se ubica según cuánto creció la open de la referencia; arriba, la mediana del
    #tiempo relativo en cada tamaño; abajo, cuántos caminos hay de cada tamaño
    base_label, base_color = styles[BASELINE]
    others = {s: v for s, v in styles.items() if s != BASELINE}
    if not others:
        return
    capacity = ((data["open_bytes_base"] - BASELINE_STRUCT_BYTES) / BASELINE_NODE_BYTES).round().astype(int)
    df = data.assign(open_bin=capacity.clip(lower=OPEN_SMALLEST_BIN))
    counts = df[df["structure"] == BASELINE].groupby("open_bin").size()
    bins = [b for b, n in counts.items() if n >= MIN_PER_OPEN_BIN]
    labels = [f"≤{fmt_number(b)}" if b == OPEN_SMALLEST_BIN else fmt_number(b) for b in bins]
    x = np.arange(len(bins))

    table = df[df["open_bin"].isin(bins)].pivot_table(index="open_bin", columns="structure", values="time_ratio",
                                                       aggfunc="median", observed=True)
    table.insert(0, "caminos", counts[bins])
    table.index.name = "max_nodos_open_redondeado"
    table.rename(columns=lambda s: styles[s][0] if s in styles else s).to_csv(OUT_DIR / "resumen_por_open.csv")

    fig, (ax, ax_n) = plt.subplots(2, 1, figsize=(11, 9.2), sharex=True, gridspec_kw={"height_ratios": [2.2, 1]})
    ax.axhline(1, color=base_color, linewidth=1.2, zorder=1, label=f"{base_label} (referencia = 1×)")
    drawn, ends = {}, []
    for structure, (label, color) in others.items():
        y = table[structure].to_numpy()
        plot_series(ax, drawn, label, x, y, color, label, lambda other, label=label: f"{label}\n(igual a {other})")
        #con pocos tamaños, un punto por tamaño ayuda a leerlo; el borde del color de fondo los separa si se cruzan
        ax.plot(x, y, linestyle="none", marker="o", markersize=6.5, color=color,
                markeredgecolor=INK["surface"], markeredgewidth=1.2, zorder=4)
        ends.append((x[-1], y[-1], label))
    ax.set_title(f"Tiempo relativo a {base_label} (mediana por camino)")
    ax.set_ylabel(f"Veces el tiempo de {base_label}")
    keep_reference_visible(ax)
    ax.set_xlim(-0.4, len(bins) - 1 + 0.9)
    #sharex esconde las etiquetas del eje x en el panel de arriba; aquí se vuelven a mostrar
    ax.set_xticks(x, labels)
    ax.tick_params(labelbottom=True)
    ax.set_xlabel(f"Máximo de nodos en la open de {base_label} (redondeado a potencia de 2)")
    #con un solo panel ancho, la leyenda cabe en una fila
    panel_legend(ax, ncol=len(others) + 1)
    add_end_labels(ax, ends)

    ax_n.bar(x, counts[bins].to_numpy(), width=0.24, color=base_color)
    ax_n.set_title("Cuántos caminos llegan a cada tamaño de open")
    ax_n.set_ylabel("Caminos")
    ax_n.yaxis.set_major_formatter(FuncFormatter(lambda v, _: fmt_number(v)))
    ax_n.set_xticks(x, labels)
    ax_n.tick_params(labelbottom=True)
    ax_n.set_xlabel(f"Máximo de nodos en la open de {base_label} (redondeado a potencia de 2)")

    fig.suptitle("Todos los mapas: tiempo según el tamaño de la open", x=0.012, ha="left",
                 fontsize=14, color=INK["primary"], y=0.99)
    fig.text(0.012, 0.905,
             f"Cada camino se ubica según el máximo de nodos que tuvo a la vez la open de {base_label}. Bajo 1× la estructura gana.\n"
             "Los tamaños grandes son pocos caminos, pero son los más largos y los que más tiempo toman.\n"
             f"El tiempo relativo solo usa caminos donde {base_label} tardó al menos {MIN_US_FOR_RATIO} µs (más cortos, el redondeo a µs los empata).",
             fontsize=9.5, color=INK["secondary"], linespacing=1.5)
    fig.tight_layout(rect=(0, 0, 1, 0.9), h_pad=2.4)
    apply_ratio_format(ax)
    out = OUT_DIR / "tiempo_por_open.png"
    fig.savefig(out, dpi=160)
    plt.close(fig)
    print(f"guardado {out}")


def label_bars(ax, x, values, texts, baseline=0):
    #el número va en la punta de cada barra, en tinta de texto y no en el color de la serie;
    #una barra que baja de la línea base (gana contra la referencia) lo lleva bajo la punta
    for xi, value, text in zip(x, values, texts):
        down = value < baseline
        ax.annotate(text, xy=(xi, value), xytext=(0, -4 if down else 4), textcoords="offset points",
                    ha="center", va="top" if down else "bottom", fontsize=8.5, color=INK["secondary"])


def plot_by_difficulty(data, styles, levels, value_col, ratio_col, title, unit, what, note, out_name):
    #un panel por nivel de dificultad, con una barra por estructura:
    #- arriba, la mediana por camino de cada una. Cada panel tiene su propia escala, porque una ruta
    #  difícil cuesta órdenes de magnitud más que una fácil y en una escala común las fáciles no se verían
    #- abajo, la mediana del cociente contra la referencia. Las barras salen de 1× (bajo 1× gana) y los
    #  tres paneles comparten escala, para ver cómo cambia la ventaja con la dificultad
    base_label, base_color = styles[BASELINE]
    names = list(styles)
    others = [s for s in names if s != BASELINE]
    if not others:
        return
    values = data.pivot_table(index="difficulty", columns="structure", values=value_col,
                              aggfunc="median", observed=True)
    ratios = data.pivot_table(index="difficulty", columns="structure", values=ratio_col,
                              aggfunc="median", observed=True)

    fig, axes = plt.subplots(2, len(levels), figsize=(15, 9.8))
    for ax in axes[1, 1:]:
        ax.sharey(axes[1, 0])
    #itertuples y no iterrows: iterrows pasa todas las columnas a float y los conteos saldrían "156.0"
    for col, row in enumerate(levels.itertuples()):
        level = row.Index
        ax_top, ax_bottom = axes[0, col], axes[1, col]

        x = np.arange(len(names))
        y = values.loc[level, names].to_numpy()
        #el borde del color de fondo deja un espacio entre barras vecinas
        ax_top.bar(x, y, width=0.72, color=[styles[s][1] for s in names],
                   edgecolor=INK["surface"], linewidth=1, zorder=2)
        label_bars(ax_top, x, y, [fmt_value(v) for v in y])
        ax_top.set_ylim(0, y.max() * 1.15)
        ax_top.set_xticks(x, [styles[s][0] for s in names], rotation=25, ha="right")
        #cada nivel dice qué largos abarca y de dónde salen sus rutas: con cortes absolutos,
        #las rutas difíciles vienen solo de los mapas grandes
        ax_top.set_title(f"{level}: largo {fmt_number(row.largo_min)} a {fmt_number(row.largo_max)}\n"
                         f"{fmt_number(row.caminos)} caminos de {fmt_number(row.mapas)} mapas")

        x = np.arange(len(others))
        y = ratios.loc[level, others].to_numpy()
        ax_bottom.axhline(1, color=base_color, linewidth=1.2, zorder=3)
        ax_bottom.bar(x, y - 1, bottom=1, width=0.72, color=[styles[s][1] for s in others],
                      edgecolor=INK["surface"], linewidth=1, zorder=2)
        label_bars(ax_bottom, x, y, [fmt_number(v, 2) + "×" for v in y], baseline=1)
        ax_bottom.set_xticks(x, [styles[s][0] for s in others], rotation=25, ha="right")
        ax_bottom.set_title(f"{level}: relativo a {base_label} (línea = 1×)")
        if col > 0:
            #comparten escala con el primer panel de la fila, así que sus números sobran
            ax_bottom.tick_params(labelleft=False)
    axes[0, 0].set_ylabel(unit)
    axes[1, 0].set_ylabel(f"Veces {what} de {base_label}")

    #espacio para los números solo del lado donde hay barras: bajo 1× si alguna gana, sobre 1× si alguna
    #pierde; del otro lado basta un margen chico (en memoria casi todo queda sobre 1× y el eje no debe bajar de 0)
    all_ratios = ratios.loc[levels.index, others].to_numpy()
    low, high = min(np.nanmin(all_ratios), 1), max(np.nanmax(all_ratios), 1)
    span = high - low
    bottom = low - (0.2 if low < 1 else 0.06) * span
    top = high + (0.2 if high > 1 else 0.06) * span
    axes[1, 0].set_ylim(max(bottom, 0), top)

    fig.suptitle(f"Todos los mapas: {title} según la dificultad de la ruta", x=0.012, ha="left",
                 fontsize=14, color=INK["primary"], y=0.99)
    fig.text(0.012, 0.9,
             "La dificultad sale del largo del camino óptimo (el bucket de MovingAI); los cortes son los terciles de todos los "
             "escenarios, los mismos para todos los mapas.\n"
             "Arriba, la mediana por camino de cada estructura (cada panel con su escala). Abajo, la mediana de cada camino "
             f"dividido por ese mismo camino en {base_label},\nque no es el cociente de las barras de arriba. {note}",
             fontsize=9.5, color=INK["secondary"], linespacing=1.5)
    fig.tight_layout(rect=(0, 0, 1, 0.885), h_pad=2.6, w_pad=2.5)
    for ax in axes[0]:
        apply_tick_format(ax)
    apply_ratio_format(axes[1, 0])
    out = OUT_DIR / out_name
    fig.savefig(out, dpi=160)
    plt.close(fig)
    print(f"guardado {out}")


if __name__ == "__main__":
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    data = with_difficulty(load_results())
    styles = structure_styles(sorted(data["structure"].unique()))
    print_optimality(data, styles)
    save_summaries(data, styles)
    levels = save_difficulty_summary(data, styles)
    for map_name in MAPS_TO_PLOT:
        plot_map(data, map_name, styles)
    plot_global(data, styles)
    plot_by_open(data, styles)
    base_label = styles[BASELINE][0]
    plot_by_difficulty(data, styles, levels, "time_ms", "time_ratio", "tiempo de ejecución",
                       "Milisegundos por camino", "el tiempo",
                       f"Bajo 1× la estructura gana. El tiempo relativo solo usa caminos donde {base_label} "
                       f"tardó al menos {MIN_US_FOR_RATIO} µs.",
                       "tiempo_por_dificultad.png")
    plot_by_difficulty(data, styles, levels, "open_kb", "memory_ratio", "memoria de la open",
                       "Kilobytes por camino", "la memoria",
                       "Bajo 1× la estructura gana. La memoria es lo que reservaba la open al llegar a la meta.",
                       "memoria_por_dificultad.png")
