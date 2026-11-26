use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::mem::size_of;

use crate::a_star::SearchNode;
use crate::open_list::OpenList;

//Medición del máximo de memoria que reserva la open durante la búsqueda.
//
//size_in_bytes() dice cuánto reserva la open en el momento en que se llama, y A* lo llama al
//llegar a la meta. Para BinaryHeap y los radix eso también es su máximo, porque nunca devuelven
//memoria, pero el vEB sí la devuelve (al vaciarse un grupo de empates o un cluster) y al llegar
//puede tener bastante menos que en su peor momento.
//Recalcular size_in_bytes() después de cada operación serviría, pero en el vEB recorre todo el
//árbol y hace la búsqueda ~3 veces más lenta. En vez de eso se cuenta lo que se pide al allocator.

//Bytes que este hilo tiene pedidos al allocator en este momento. Es isize porque la memoria que
//pide un hilo y libera otro lo dejaría negativo (este programa usa un solo hilo, pero por si acaso).
thread_local! {
    static LIVE_BYTES: Cell<isize> = const { Cell::new(0) };
}

fn add_live_bytes(delta: isize) {
    //try_with no falla si el hilo ya está terminando; en ese caso no hay nada que medir
    let _ = LIVE_BYTES.try_with(|bytes| bytes.set(bytes.get() + delta));
}

fn live_bytes() -> isize {
    LIVE_BYTES.try_with(Cell::get).unwrap_or(0)
}

//Allocator que reemplaza al de Rust en todo el programa: cada Vec, HashMap o Box pide y devuelve
//memoria a través de él. No cambia cómo se reserva (le pasa todo al allocator del sistema),
//solo lleva la cuenta en LIVE_BYTES.
struct CountingAllocator;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            add_live_bytes(layout.size() as isize);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            add_live_bytes(layout.size() as isize);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        add_live_bytes(-(layout.size() as isize));
    }

    //agrandar o achicar un bloque (lo que hace un Vec al crecer): cuenta solo la diferencia
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            add_live_bytes(new_size as isize - layout.size() as isize);
        }
        new_ptr
    }
}

//Envoltorio que mide cualquier open sin cambiarla: le delega todo y, alrededor de cada operación,
//mira cuánto cambió LIVE_BYTES. Ese cambio es justo lo que la open pidió (o devolvió) en esa
//operación, porque mientras corre su insert o su pop_min nada más pide memoria.
//Se usa como a_star::<Medido<VebOpen>, _>(...)
pub struct Medido<Open> {
    open: Open,
    //bytes que la open tiene pedidos ahora, y el máximo que llegó a tener
    bytes: isize,
    peak: isize,
}

impl<Open: OpenList> Medido<Open> {
    //corre una operación de la open y suma a bytes lo que cambió la memoria pedida
    fn medir<R>(&mut self, operation: impl FnOnce(&mut Open) -> R) -> R {
        let before = live_bytes();
        let result = operation(&mut self.open);
        self.bytes += live_bytes() - before;
        self.peak = self.peak.max(self.bytes);
        result
    }

    //máximo reservado durante la búsqueda, comparable con size_in_bytes: el struct más lo pedido
    pub fn peak_bytes(&self) -> usize {
        size_of::<Open>() + self.peak as usize
    }
}

impl<Open: OpenList> Default for Medido<Open> {
    //crear la open también se mide: el radix heap del crate ya pide su arreglo de buckets aquí
    fn default() -> Self {
        let before = live_bytes();
        let open = Open::default();
        let bytes = live_bytes() - before;
        Self { open, bytes, peak: bytes }
    }
}

impl<Open: OpenList> OpenList for Medido<Open> {
    //el mismo nombre de la open de adentro, así los resultados van a la misma carpeta
    const NAME: &'static str = Open::NAME;

    fn insert(&mut self, node: SearchNode) {
        self.medir(|open| open.insert(node));
    }

    //sacar también puede pedir memoria: los radix heaps mueven nodos a otros buckets al
    //reorganizarse, y RadixAltRound guarda el espacio liberado en free_slots
    fn pop_min(&mut self) -> Option<SearchNode> {
        self.medir(|open| open.pop_min())
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

//Envoltorio para la búsqueda limpia, la que da el tiempo total: deja pasar todo a la open, salvo
//los tamaños que A* le pide al llegar a la meta (size_in_bytes y used_bytes), que devuelve en 0.
//En el vEB calcularlos recorre todos sus grupos y su árbol, y ese recorrido quedaría dentro del
//tiempo total, más largo mientras más grande la open: el vEB saldría más lento por medir su memoria.
//Los tamaños se toman de la pasada con cronómetro, que hace la misma búsqueda y calcula esos
//tamaños fuera de sus intervalos (ver run_map_struct_save en testing_functions.rs)
pub struct SinTamanos<Open> {
    open: Open,
}

impl<Open> SinTamanos<Open> {
    //la open de adentro, por ejemplo para leer el máximo de memoria que midió Medido
    pub fn inner(&self) -> &Open {
        &self.open
    }
}

impl<Open: OpenList> Default for SinTamanos<Open> {
    fn default() -> Self {
        Self { open: Open::default() }
    }
}

impl<Open: OpenList> OpenList for SinTamanos<Open> {
    const NAME: &'static str = Open::NAME;

    fn insert(&mut self, node: SearchNode) {
        self.open.insert(node);
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        self.open.pop_min()
    }

    fn size_in_bytes(&self) -> usize {
        0
    }

    fn used_bytes(&self) -> usize {
        0
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.open.nodes()
    }
}
