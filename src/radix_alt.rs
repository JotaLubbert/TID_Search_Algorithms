use std::mem::size_of;

//Radix heap monótono de mínimos sobre claves u64, escrito aparte para no tocar el crate
//radix-heap (así el radix original se sigue midiendo tal cual). Cambios respecto del crate:
//- Una máscara de bits marca qué buckets tienen elementos. Cuando el bucket 0 se vacía,
//  el siguiente con elementos sale de un trailing_zeros, en vez de recorrer los 65.
//- Es de mínimos directamente (sin Reverse) y parte con top = 0, así que no necesita
//  el bucket "initial" del crate: toda clave u64 es >= 0.
//- Es genérico en el valor; la open guarda un índice u32 para que cada movimiento entre
//  buckets copie 16 bytes y no los 56 de una entrada con el SearchNode completo.
//
//Funcionamiento: top es la última clave extraída y toda clave nueva debe ser >= top.
//Una clave k va al bucket del bit más alto en que difiere de top (0 si k == top).
//Al vaciarse el bucket 0, el primer bucket con elementos contiene el siguiente mínimo:
//ese mínimo pasa a ser top y los elementos del bucket se reparten en buckets más bajos.

const BUCKETS: usize = 65;

pub struct RadixHeap<V> {
    len: usize,
    top: u64,
    buckets: [Vec<(u64, V)>; BUCKETS],
    //bit i encendido = el bucket i tiene elementos
    nonempty: u128,
}

//bucket de una clave respecto de top: 0 si son iguales, si no 1 + el bit más alto en que difieren
fn bucket_index(key: u64, top: u64) -> usize {
    (64 - (key ^ top).leading_zeros()) as usize
}

impl<V> RadixHeap<V> {
    pub fn new() -> Self {
        Self { len: 0, top: 0, buckets: std::array::from_fn(|_| Vec::new()), nonempty: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    //última clave extraída; las claves nuevas no pueden ser menores
    pub fn top(&self) -> u64 {
        self.top
    }

    pub fn push(&mut self, key: u64, value: V) {
        assert!(key >= self.top, "la clave {key} es menor que la última extraída ({})", self.top);
        let bucket = bucket_index(key, self.top);
        self.buckets[bucket].push((key, value));
        self.nonempty |= 1u128 << bucket;
        self.len += 1;
    }

    //saca un elemento de clave mínima; entre claves iguales sale primero el último insertado
    pub fn pop(&mut self) -> Option<(u64, V)> {
        if self.buckets[0].is_empty() {
            if self.nonempty == 0 {
                return None;
            }
            //el bit 0 está apagado, así que esto es el primer bucket con elementos
            let source = self.nonempty.trailing_zeros() as usize;
            //todos los elementos de source van a buckets más bajos: se separan con split_at_mut
            //para vaciar source sin perder su capacidad, igual que el crate
            let (lower, rest) = self.buckets.split_at_mut(source);
            let moved = &mut rest[0];
            let new_top = moved.iter().map(|&(key, _)| key).min().expect("bucket marcado como no vacío");
            let mut mask = 0u128;
            for (key, value) in moved.drain(..) {
                let bucket = bucket_index(key, new_top);
                debug_assert!(bucket < source);
                lower[bucket].push((key, value));
                mask |= 1u128 << bucket;
            }
            self.top = new_top;
            self.nonempty = (self.nonempty & !(1u128 << source)) | mask;
        }
        let item = self.buckets[0].pop();
        if self.buckets[0].is_empty() {
            self.nonempty &= !1;
        }
        self.len -= 1;
        item
    }

    //recorre los valores sin importar el orden
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.buckets.iter().flat_map(|bucket| bucket.iter().map(|(_, value)| value))
    }

    //memoria que piden al allocator los 65 buckets (sus Vec nunca devuelven capacidad);
    //el arreglo de buckets en sí va dentro del struct y se cuenta con size_of
    pub fn heap_bytes(&self) -> usize {
        self.buckets.iter().map(|bucket| bucket.capacity() * size_of::<(u64, V)>()).sum()
    }
}

impl<V> Default for RadixHeap<V> {
    fn default() -> Self {
        Self::new()
    }
}
