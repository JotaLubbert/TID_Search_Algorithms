use image::{RgbImage, Rgb};
use imageproc::drawing::draw_line_segment_mut;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use crate::CustomMap;
use crate::a_star::SearchNode;
type Coords = (u32, u32);
type Distance = f64;

const TRANSITABLE: Rgb<u8> = Rgb([255, 255, 255]);
const BLOQUEADO: Rgb<u8>   = Rgb([160, 160, 160]);
const OPEN_COLOR: Rgb<u8>  = Rgb([144, 238, 144]);
const CLOSE_COLOR: Rgb<u8> = Rgb([0, 100, 0]);
const START_COLOR: Rgb<u8> = Rgb([0, 0, 255]);
const GOAL_COLOR: Rgb<u8>  = Rgb([255, 0, 0]);
const PATH_COLOR: Rgb<u8>  = Rgb([255, 200, 0]);
const GRID_COLOR: Rgb<u8>  = Rgb([210, 210, 210]);

pub fn visualize_final_state(
    map: & CustomMap,
    open: &BinaryHeap<Reverse<SearchNode>>,
    close: &HashMap<Coords, SearchNode>,
    start: Coords,
    goal: Coords,
    path: &[Coords],
    scale: u32,
    output_path: &str,
) {
    let path_thickness = 1;
    let grid_size = 512u32;
    let img_size = grid_size * scale;
    let mut img = RgbImage::new(img_size, img_size);

    let mut fill_cell = |img: &mut RgbImage, coord: Coords, color: Rgb<u8>| {
        let (cx, cy) = coord;
        for dy in 0..scale {
            for dx in 0..scale {
                img.put_pixel(cx * scale + dx, cy * scale + dy, color);
            }
        }
    };

    // 1. base: transitable / bloqueado
    for y in 0..grid_size {
        for x in 0..grid_size {
            let color = if map[y as usize][x as usize] { TRANSITABLE } else { BLOQUEADO };
            fill_cell(&mut img, (x, y), color);
        }
    }

    // 2. close (verde oscuro)
    for coord in close.keys() {
        fill_cell(&mut img, *coord, CLOSE_COLOR);
    }

    // 3. open (verde claro)
    for Reverse(node) in open.iter() {
        fill_cell(&mut img, node.coords, OPEN_COLOR);
    }

    // 6. start y goal, siempre visibles por encima de todo
    fill_cell(&mut img, start, START_COLOR);
    fill_cell(&mut img, goal, GOAL_COLOR);

    // 4. grilla — se dibuja DESPUÉS de los colores de fondo, pero ANTES del path,
    // así el path queda por encima de las líneas de grilla, no tapado por ellas    
    draw_grid(&mut img, grid_size, scale);

    // 5. línea del camino final, con grosor
    draw_thick_path(&mut img, path, scale, path_thickness, PATH_COLOR);

    img.save(output_path).expect("no se pudo guardar la imagen");
}

/// Dibuja líneas horizontales y verticales cada `scale` pixeles,
/// marcando el borde de cada celda del grid original.
fn draw_grid(img: &mut RgbImage, grid_size: u32, scale: u32) {
    let img_size = (grid_size * scale) as f32;
    for i in 0..=grid_size {
        let pos = (i * scale) as f32;
        draw_line_segment_mut(img, (pos, 0.0), (pos, img_size), GRID_COLOR);
        draw_line_segment_mut(img, (0.0, pos), (img_size, pos), GRID_COLOR);
    }
}

/// Dibuja el camino como una línea gruesa, trazando múltiples líneas
/// paralelas desplazadas perpendicularmente al segmento original.
fn draw_thick_path(img: &mut RgbImage, path: &[Coords], scale: u32, thickness: u32, color: Rgb<u8>) {
    let half = scale as f32 / 2.0;
    let half_thickness = thickness as f32 / 2.0;

    for window in path.windows(2) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        let p1 = ((x1 * scale) as f32 + half, (y1 * scale) as f32 + half);
        let p2 = ((x2 * scale) as f32 + half, (y2 * scale) as f32 + half);

        // vector perpendicular al segmento, normalizado
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let perp_x = -dy / len;
        let perp_y = dx / len;

        // dibujamos varias líneas paralelas desplazadas a lo largo de la perpendicular
        let steps = thickness.max(1);
        for i in 0..steps {
            let offset = -half_thickness + i as f32;
            let ox = perp_x * offset;
            let oy = perp_y * offset;
            draw_line_segment_mut(
                img,
                (p1.0 + ox, p1.1 + oy),
                (p2.0 + ox, p2.1 + oy),
                color,
            );
        }
    }
}
