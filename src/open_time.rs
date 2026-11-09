use std::cell::RefCell;

use crate::a_star::SearchNode;
use crate::open_list::OpenList;

//Medición del tiempo que usa la open dentro de A*, por resta (columna open_time_ns de los TSV).
//
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
