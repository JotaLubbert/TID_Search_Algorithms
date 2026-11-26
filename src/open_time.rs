use std::cell::RefCell;
use std::hint::black_box;
use std::time::Instant;

use crate::a_star::SearchNode;
use crate::open_list::OpenList;

//Dos formas de medir el tiempo que usa la open:
//- con cronómetro en cada operación (Cronometrado, al final de este archivo): es la que se usa en
//  el informe (columnas open_timer_ns, open_intervals y clock_overhead_ns de los TSV)
//- por resta (Grabado y Repeticion, aquí abajo): queda para el paper. Hoy no se llama
//
//Medición por resta.
//Cronometrar cada insert y pop_min no sirve: leer el reloj cuesta ~18 ns y una operación de la open
//dura entre 16 y 37 ns, así que la medición pesa tanto como lo medido (alargaba A* entre 27% y 42%).
//En vez de eso, cada búsqueda se corre dos veces (run_map_struct_save en testing_functions.rs):
//1. la búsqueda cronometrada de siempre, con la open envuelta en Grabado, que anota qué nodos
//   devolvió pop_min y en qué orden
//2. con Repeticion, una open falsa que devuelve esos mismos nodos en el mismo orden sin ordenar nada
//A* hace exactamente el mismo trabajo fuera de la open en las dos (mismas expansiones, mismos
//sucesores, mismo close), así que la diferencia de tiempo es lo que costó la open, incluido lo que
//su uso de la caché le cuesta al resto de A*.
//Anotar un nodo (un push en un Vec) cuesta lo mismo que leerlo en la repetición, así que ese costo
//se cancela en la resta.

thread_local! {
    //la grabación pasa por aquí porque A* crea la open con Default y no hay otra forma de darle datos.
    //Además guarda la capacidad entre búsquedas: así grabar no pide memoria durante la búsqueda real
    static CINTA: RefCell<Vec<SearchNode>> = const { RefCell::new(Vec::new()) };
}

//Envoltorio que deja pasar todo a la open real y anota cada nodo que sale con pop_min
#[allow(dead_code)] //se usa en la medición por resta, que queda para el paper
pub struct Grabado<Open> {
    open: Open,
    sacados: Vec<SearchNode>,
}

impl<Open: OpenList> Default for Grabado<Open> {
    fn default() -> Self {
        //reutiliza el Vec (y su capacidad) de la búsqueda anterior
        let mut sacados = CINTA.with(|cinta| std::mem::take(&mut *cinta.borrow_mut()));
        sacados.clear();
        Self { open: Open::default(), sacados }
    }
}

#[allow(dead_code)] //se usa en la medición por resta, que queda para el paper
impl<Open> Grabado<Open> {
    //la open de adentro, por ejemplo para leer el máximo de memoria que midió Medido
    pub fn inner(&self) -> &Open {
        &self.open
    }

    //deja la grabación lista para que la tome la próxima Repeticion
    pub fn preparar_repeticion(&mut self) {
        CINTA.with(|cinta| *cinta.borrow_mut() = std::mem::take(&mut self.sacados));
    }
}

impl<Open: OpenList> OpenList for Grabado<Open> {
    const NAME: &'static str = Open::NAME;

    fn insert(&mut self, node: SearchNode) {
        self.open.insert(node);
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        let node = self.open.pop_min()?;
        self.sacados.push(node);
        Some(node)
    }

    fn size_in_bytes(&self) -> usize {
        self.open.size_in_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.open.used_bytes()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.open.nodes()
    }
}

//Open falsa: devuelve la grabación en orden y no guarda lo que A* le inserta. Como sale lo mismo
//que en la búsqueda real, A* toma las mismas decisiones (cuáles descartar, cuáles expandir)
#[allow(dead_code)] //se usa en la medición por resta, que queda para el paper
pub struct Repeticion {
    sacados: Vec<SearchNode>,
    siguiente: usize,
}

impl Default for Repeticion {
    fn default() -> Self {
        Self { sacados: CINTA.with(|cinta| std::mem::take(&mut *cinta.borrow_mut())), siguiente: 0 }
    }
}

impl Drop for Repeticion {
    //devuelve el Vec para que el próximo Grabado reutilice su capacidad
    fn drop(&mut self) {
        CINTA.with(|cinta| *cinta.borrow_mut() = std::mem::take(&mut self.sacados));
    }
}

impl OpenList for Repeticion {
    const NAME: &'static str = "repeticion";

    fn insert(&mut self, _node: SearchNode) {}

    fn pop_min(&mut self) -> Option<SearchNode> {
        let node = self.sacados.get(self.siguiente).copied();
        self.siguiente += 1;
        node
    }

    fn size_in_bytes(&self) -> usize {
        0
    }

    fn used_bytes(&self) -> usize {
        0
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.sacados.iter().skip(self.siguiente)
    }
}

//Medición con cronómetro (la del informe).
//
//La open se cronometra operación por operación en una segunda pasada de A*, aparte de la búsqueda
//limpia que da el tiempo total: así el cronómetro no toca ese tiempo. Cada intervalo arrastra parte
//de la lectura del reloj (~18 ns contra 16–37 ns de una operación), así que justo antes de cada
//pasada se mide cuánto dura un intervalo vacío (calibrar_reloj) y se guardan la suma cruda de los
//intervalos, cuántos son y esa calibración: el análisis resta intervalos × calibración.
//Lo que no se corrige: leer el reloj obliga al procesador a terminar lo anterior, así que la open no
//se solapa con el resto de A* como en la búsqueda limpia; y lo que la open le cuesta al resto de A*
//en la caché queda fuera de los intervalos. La medición por resta sí incluye las dos cosas.

//Envoltorio que cronometra cada operación de la open real
pub struct Cronometrado<Open> {
    open: Open,
    //suma de los intervalos en ns y cuántos son (crear la open cuenta como uno)
    ns: u64,
    intervalos: u64,
}

impl<Open> Cronometrado<Open> {
    pub fn ns(&self) -> u64 {
        self.ns
    }

    pub fn intervalos(&self) -> u64 {
        self.intervalos
    }
}

impl<Open: OpenList> Default for Cronometrado<Open> {
    //crear la open también se cronometra: A* la crea dentro de la búsqueda, y el radix heap del
    //crate ya pide su arreglo de buckets aquí
    fn default() -> Self {
        let inicio = Instant::now();
        let open = Open::default();
        let ns = inicio.elapsed().as_nanos() as u64;
        Self { open, ns, intervalos: 1 }
    }
}

impl<Open: OpenList> OpenList for Cronometrado<Open> {
    const NAME: &'static str = Open::NAME;

    fn insert(&mut self, node: SearchNode) {
        let inicio = Instant::now();
        self.open.insert(node);
        self.ns += inicio.elapsed().as_nanos() as u64;
        self.intervalos += 1;
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        let inicio = Instant::now();
        let node = self.open.pop_min();
        self.ns += inicio.elapsed().as_nanos() as u64;
        self.intervalos += 1;
        node
    }

    fn size_in_bytes(&self) -> usize {
        self.open.size_in_bytes()
    }

    fn used_bytes(&self) -> usize {
        self.open.used_bytes()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.open.nodes()
    }
}

//cuánto mide en ns un intervalo vacío: el mismo par de lecturas del reloj que rodea cada operación,
//sin nada adentro. Se toma justo antes de cada pasada cronometrada porque depende de la frecuencia
//de la CPU en ese momento. Es el promedio de la mitad central de 2.001 intervalos: así no cuentan
//las interrupciones y quedan decimales (el reloj da ns enteros)
pub fn calibrar_reloj() -> f64 {
    let mut vacios = [0u64; 2001];
    for vacio in &mut vacios {
        let inicio = Instant::now();
        *vacio = black_box(inicio).elapsed().as_nanos() as u64;
    }
    vacios.sort_unstable();
    let centro = &vacios[500..1501];
    centro.iter().sum::<u64>() as f64 / centro.len() as f64
}
