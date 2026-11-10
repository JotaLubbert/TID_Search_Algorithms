# Desempate en la open: por qué LIFO y no "mayor g"

## Resumen

Para comparar las estructuras de la open, todas deben expandir exactamente los mismos nodos. Si no,
parte de la diferencia de tiempo viene de cuántos nodos recorre A\* y no de la estructura. Para eso,
todas tienen que resolver los empates de f con la misma regla. Se eligió:

> **f redondeada a 2⁻¹⁸ y, entre nodos con la misma f, sale primero el último que entró (LIFO).**

Las razones:

1. **Es la única regla que todas las estructuras pueden aplicar de forma exacta.** "Mayor g primero"
   es imposible en un radix heap, porque obligaría a insertar claves menores que la última extraída
   (sección 3). Tres de las siete estructuras son radix heaps.
2. **No cuesta expansiones:** en los mapas medidos, LIFO expande entre 0 % y 0,9 % *menos* que
   "mayor g" (sección 6).
3. **RadixAltRound ya la cumple:** sus búsquedas son idénticas, camino por camino, a las de un
   BinaryHeap con esta regla explícita (sección 6).

## 1. Por qué importan los empates

Con la heurística octil, muchísimos nodos tienen la misma f: en este proyecto, entre el 75 % y el 84 %
de los `pop_min` sacan un nodo con la misma f que el anterior. Todas las estructuras sacan siempre la
menor f, así que lo único que cambia qué nodos se expanden es **el orden entre empates**.

Hoy cada estructura desempata a su manera, y eso se nota. En las rutas fáciles, QuickHeap expande
un 21 % más de nodos que BinaryHeap (877 contra 724 por camino) y tarda un 18 % más. Ese tiempo
extra no viene de su open, sino de que recorre más nodos.

## 2. Las reglas posibles

| Regla | Entre nodos con la misma f, sale primero… |
|---|---|
| Mayor g (o menor h, que es lo mismo con f igual) | el más avanzado desde el inicio |
| LIFO | el último que entró a la open |
| FIFO | el primero que entró |
| Sin regla (hoy) | lo que resulte del orden interno de cada estructura |

"Mayor g" es la recomendación habitual en la literatura: entre nodos igual de prometedores, conviene
el que está más cerca de la meta. LIFO se le parece en la práctica: el último en entrar suele ser un
sucesor del último expandido, o sea, un nodo un paso más adelante.

## 3. Por qué no "mayor g": los radix heaps no pueden aplicarla

Un radix heap (RadixHeap, RadixAlt y RadixAltRound) tiene una condición: **toda clave que se inserta
debe ser mayor o igual que la última que se extrajo**. En eso se basa su eficiencia, porque reparte
las claves en buckets según cuánto difieren de esa última.

Para desempatar por mayor g, la clave tendría que ser el par (f, −g). Con una heurística consistente
como la octil, esto rompe la condición. Un ejemplo:

- Se extrae el nodo **n** con f = 10, g = 4 (h = 6). Su clave es (10, −4).
- Su sucesor **m**, un paso hacia la meta, tiene g = 5 y h = 5, o sea f = 10. Su clave es (10, −5).
- (10, −5) es **menor** que (10, −4), la última extraída: el radix heap no puede aceptarla.

Esto ocurre todo el tiempo, porque es justo el caso de los empates: los sucesores que avanzan hacia la
meta tienen la misma f y mayor g. Nuestras implementaciones de radix, cuando reciben una clave menor
que la última, la suben a ese valor (`key.max(top)`). Eso hace que el nodo salga de inmediato: es
exactamente LIFO, no "mayor g".

BinaryHeap, QuickHeap y el vEB sí podrían ordenar por mayor g, pero entonces los radix heaps
quedarían con otra regla y las búsquedas volverían a ser distintas.

## 4. Por qué LIFO sí funciona en todas

Con LIFO, la clave es solo la f redondeada. Un sucesor con la misma f tiene la misma clave que la
última extraída, y eso sí está permitido en un radix heap. El orden entre empates lo da el orden de
llegada:

| Estructura | Cómo aplica LIFO |
|---|---|
| RadixHeap, RadixAlt y RadixAltRound | los empates quedan en el bucket de la clave actual, que se vacía como una pila |
| vEB | los nodos con la misma f se guardan en una pila por clave |
| BinaryHeap | la clave pasa a ser (f redondeada, contador de inserciones invertido) |
| QuickHeap y QuickHeapScalar | el contador de inserciones invertido va en los 32 bits bajos de la clave |

### Por qué redondear la f

Dos f matemáticamente iguales pueden diferir en el último decimal, según el orden en que se sumaron
los pasos de g. Con la clave exacta (`f.to_bits()`) esos nodos no empatan, y el orden lo decide ese
ruido. Por eso RadixAlt, RadixHeap y el vEB expanden distinto que RadixAltRound, aunque los tres sean
LIFO.

Redondear a 2⁻¹⁸ (262.144 segmentos por unidad de f, cada uno de ≈ 0,0000038) elimina ese ruido sin
juntar f distintas. Con la octil, toda f es P + Q·√2, con P y Q enteros. Hasta f ≈ 7.600, dos f
distintas se separan por ≈ 0,00015 o más, unas 39 veces el segmento. El redondeo mueve cada clave como
mucho medio segmento, así que nunca junta ni invierte dos f distintas.

Elegimos 2⁻¹⁸ y no 2⁻²⁰ por la colección Mazes (60 mapas de 512×512). Su camino óptimo más largo cuesta
7.583 (maze512-1-4), contra 2.827,6 en DragonAges (orz900d). Con 2⁻²⁰ el assert de QuickHeap falló en
esos laberintos con f = 4.096,07.

## 5. Por qué redondear en todas las estructuras

¿Y si no redondeamos y solo aplicamos LIFO con la clave exacta `f.to_bits()`? No alcanza.

**LIFO solo no basta.** Con la clave exacta, el ruido de punto flotante a veces produce una clave apenas
menor que la última extraída: una f matemáticamente igual que salió con el último bit más chico. Los
radix heaps no aceptan claves menores, así que las suben al valor de la última extraída (`key.max(top)`)
y las tratan como empate. Las demás estructuras no hacen ese ajuste.

Se midió en 4 mapas el porcentaje de caminos idénticos (expansiones y camino) a RadixAlt (clave exacta
+ LIFO + ajuste):

| Mapa | vEB actual (clave exacta + LIFO, sin ajuste) | Clave exacta + LIFO + el mismo ajuste |
|---|---|---|
| arena2 | 32,9 % | 100 % |
| den312d | 71,0 % | 100 % |
| hrt201n | 44,4 % | 100 % |
| lak504d | 46,2 % | 100 % |

Un BinaryHeap con clave exacta + LIFO sin ajuste da exactamente los mismos porcentajes que el vEB.

Sin redondear, se podría unificar BinaryHeap, RadixHeap y vEB copiando el ajuste, pero QuickHeap no. Su
desempate exige poner el contador de inserciones en los 32 bits bajos del u64, así que la f debe caber
en los 32 bits altos. `f.to_bits()` usa los 64 bits y todos importan.

Además, los empates quedarían decididos por el signo del error de redondeo. Dos nodos con la misma f
matemática empatan (y sale primero el último, LIFO) si el segundo salió apenas más chico. Si salió
apenas más grande, no empatan y sale después de todos los empatados. Es un orden determinista pero
arbitrario y difícil de justificar.

El redondeo también acelera los radix heaps, porque las claves quedan más compactas. En la corrida
completa (155.620 caminos), la open de RadixAltRound fue 3,6 % más rápida que la de RadixAlt
(diferencia pareada −0,0375 ms ± 0,0009 por camino, IC 95 %). El tiempo total por camino fue 7,595 ms
contra 7,716 ms.

El redondeo es lo que permite incluir a QuickHeap y que "empate" signifique "misma f matemática", sin
costo de exactitud (sección 4).

## 6. Mediciones

**¿RadixAltRound es exactamente "f redondeada + LIFO"?** Se comparó, camino por camino, cada
estructura actual contra un BinaryHeap con esa regla explícita (mismo número de expansiones):

| Mapa | Caminos | RadixAltRound | RadixAlt / RadixHeap | vEB | BinaryHeap | QuickHeap |
|---|---|---|---|---|---|---|
| arena2 | 910 | **100 %** | 13 % | 9 % | 2 % | 2 % |
| den312d | 290 | **100 %** | 31 % | 29 % | 8 % | 9 % |
| hrt201n | 1.180 | **100 %** | 9 % | 7 % | 2 % | 2 % |
| lak504d | 720 | **100 %** | 14 % | 11 % | 2 % | 3 % |

**¿Cuánto cuesta LIFO frente a "mayor g"?** Expansiones promedio por camino. "Mayor g" se probó en un
BinaryHeap, con LIFO si también empata la g:

| Mapa | Mayor g | LIFO | BinaryHeap actual (sin regla) |
|---|---|---|---|
| arena2 | 6.441 | 6.382 (−0,9 %) | 5.991 (−7,0 %) |
| den312d | 478 | 478 (0 %) | 469 (−1,8 %) |
| hrt201n | 5.810 | 5.802 (−0,2 %) | 5.754 (−1,0 %) |
| lak504d | 3.689 | 3.678 (−0,3 %) | 3.657 (−0,9 %) |

En estos mapas, "mayor g" no expande menos que LIFO, y además genera más nodos (11.951 contra 10.841
en arena2). Elegir LIFO no tiene costo de búsqueda frente a la regla de los libros.

El orden interno actual de BinaryHeap expande algo menos, pero es un orden que ninguna otra estructura
puede reproducir: depende de cómo el heap acomoda sus elementos. Al unificar, BinaryHeap expandirá
entre 1 % y 7 % más que hoy, igual que todas las demás.

## 7. Consecuencias

- **Todas las estructuras expanden los mismos nodos y encuentran el mismo camino.** Las diferencias de
  tiempo y memoria quedan solo en la estructura.
- **RadixAlt se elimina:** con la clave redondeada sería idéntica a RadixAltRound, que conserva su nombre.
- **QuickHeap** guarda la clave en 32 bits y escala la f por 2¹⁸, igual que el resto, para que todas
  redondeen igual. Con eso la f debe ser menor que 16.384, el doble de margen sobre 7.583 (la máxima
  observada), y un assert lo vigila. Con 2²⁰ el límite era 4.096.
- **BinaryHeap** guarda el contador junto al nodo: cada entrada pasa de 48 a 64 bytes.
- **QuickHeap** necesita una tabla de contador a slot: 4 bytes por inserción.
- **Hay que volver a correr todo.**

## 8. Verificación

Antes de cambiar el código, se prueba con prototipos de las estructuras con esta regla. Cada una debe
dar **el mismo número de expansiones, el mismo número de generados y el mismo camino** que
RadixAltRound, en todos los escenarios de todos los mapas.

Para la escala 2⁻¹⁸ se comprobó que en arena2 los resultados con 2⁻¹⁸ y 2⁻²⁰ son idénticos
(expansiones, generados y camino) en BinaryHeap, RadixHeap, vEB y QuickHeap. Además, en arena2 (910
caminos) y en maze512-1-4 (474 caminos, 1 de cada 40) las 6 estructuras dan exactamente lo mismo,
incluido el camino de costo 7.570.
