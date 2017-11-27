extern crate nalgebra as na;
extern crate svg;

use std::f64::consts::PI;
use super::svg::node;
use super::svg::node::element;
use super::svg::node::element::path::Data;
use draw::consts::*;
use Orientation;
use map;
use tile;

/// Draw a hexagon
pub fn draw_hex(center: na::Vector2<f64>,
                info: &map::MapInfo) -> element::Path {
    let basis = get_basis(&info.orientation);

    let points = [
        na::Vector3::new(-1.0,  0.0,  0.0),
        na::Vector3::new( 0.0,  0.0,  1.0),
        na::Vector3::new( 0.0,  1.0,  0.0),
        na::Vector3::new( 1.0,  0.0,  0.0),
        na::Vector3::new( 0.0,  0.0, -1.0),
        na::Vector3::new( 0.0, -1.0,  0.0),
    ];
    let data = Data::new()
        .move_to(point_to_tuple(info.scale * (basis * points[0] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[1] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[2] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[3] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[4] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[5] + center)))
        .close();

    element::Path::new()
        .set("d", data)
}

/// Draws a border around a hex
pub fn draw_hex_edge(center: na::Vector2<f64>,
                     info: &map::MapInfo) -> element::Path {
    draw_hex(center, info)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH * info.scale)
}

/// Draws the background (the color) of a hex
pub fn draw_hex_background(center: na::Vector2<f64>,
                       info: &map::MapInfo,
                       color: tile::colors::Color) -> element::Path {
    draw_hex(center, info)
        .set("fill", color.value())
        .set("stroke", "none")
}

/// Draws the black inside line of a path
pub fn draw_path(path: &tile::Path,
             center: &na::Vector2<f64>,
             info: &map::MapInfo) -> element::Group {
    let mut g = element::Group::new();
    // Draw an outline if the line is a bridge
    if path.is_bridge() {
        g = g.add(draw_path_contrast(path, center, info));
    }
    g.add(draw_path_helper(path, center, info)
          .set("stroke", "black")
          .set("stroke-width", PATH_WIDTH * info.scale))
}

/// Helper for drawing the paths, does the actual point calculation
pub fn draw_path_helper(path: &tile::Path,
                        center: &na::Vector2<f64>,
                        info: &map::MapInfo) -> element::Path {
    let basis = get_basis(&info.orientation);

    // Calculate end points and control points
    let (start_x, start_y) = point_to_tuple(
        info.scale * (basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        info.scale * (basis * path.end() + center));
    let control1 = C * basis * path.start() + center;
    let control2 = C * basis * path.end() + center;
    let (x1, y1) = point_to_tuple(info.scale * control1);
    let (x2, y2) = point_to_tuple(info.scale * control2);

    // Do the drawing
    let data = Data::new()
        .move_to((start_x, start_y))
        .cubic_curve_to((x1, y1, x2, y2, end_x, end_y));
    element::Path::new()
        .set("d", data.clone())
        .set("fill", "none")
}

/// Draws the white contrast lines around a path
pub fn draw_path_contrast(path: &tile::Path,
                          center: &na::Vector2<f64>,
                          info: &map::MapInfo) -> element::Path {
    draw_path_helper(path, center, info)
        .set("stroke", "white")
        .set("stroke-width", (PATH_WIDTH + 2.0 * LINE_WIDTH) * info.scale)
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
pub fn draw_lawson(center: na::Vector2<f64>,
                   info: &map::MapInfo) -> element::Circle {
        // Add LINE_WIDTH to compensate for stroke being half in the circle
    draw_circle(&(center * info.scale),
                (PATH_WIDTH + LINE_WIDTH) * 0.5 * info.scale,
                "black", "white", LINE_WIDTH * info.scale)
}

/// Draw a city
pub fn draw_city<T>(city: tile::City,
                    center: na::Vector2<f64>,
                    info: &map::MapInfo,
                    tile: &T) -> element::Group
    where
        T: tile::TileSpec
{
    let basis = get_basis(&info.orientation);

    let text_circle_pos = info.scale * (basis * city.revenue_position()
                                        + center);
    let text_pos = text_circle_pos + info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let text = tile.text(city.text_id);
    let g = element::Group::new()
        .add(draw_circle(&text_circle_pos,
                                  REVENUE_CIRCLE_RADIUS * info.scale,
                                  "white", "black", LINE_WIDTH * info.scale))
        .add(element::Text::new()
             .add(node::Text::new(text))
             .set("x", text_pos.x)
             .set("y", text_pos.y)
             .set("style", "text-anchor:middle;"));

    let pos = info.scale * (basis * city.position() + center);
    let center = basis * city.position() + center;
    match city.circles {
        1 => g.add(draw_city_circle(&pos, info)),
        2 => {
            let pos1 = pos + na::Vector2::new(-info.scale * TOKEN_SIZE, 0.0);
            let pos2 = pos + na::Vector2::new( info.scale * TOKEN_SIZE, 0.0);
            g.add(element::Rectangle::new()
                  .set("x", (center.x - TOKEN_SIZE) * info.scale)
                  .set("y", (center.y - TOKEN_SIZE) * info.scale)
                  .set("width", TOKEN_SIZE * info.scale * 2.0)
                  .set("height", TOKEN_SIZE * info.scale * 2.0)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = info.scale * TOKEN_SIZE;
            let pos1 = pos + na::Vector2::new(0.0, -2.0 * size / sq3);
            let pos2 = pos + na::Vector2::new(-size, size / sq3);
            let pos3 = pos + na::Vector2::new( size, size / sq3);
            let data = Data::new()
                .move_to((-size + pos.x, size / sq3 + size + pos.y))
                .line_to((size + pos.x, size / sq3 + size + pos.y))
                .line_to((size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .line_to((0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .close();
            g.add(element::Path::new()
                    .set("d", data)
                    .set("fill", "white")
                    .set("stroke", "black")
                    .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
        }
        4 => {
            let size = info.scale * TOKEN_SIZE;
            let pos1 = pos + na::Vector2::new(-size, -size);
            let pos2 = pos + na::Vector2::new(-size,  size);
            let pos3 = pos + na::Vector2::new( size, -size);
            let pos4 = pos + na::Vector2::new( size,  size);
            g.add(element::Rectangle::new()
                  .set("x", (center.x - 2.0 * TOKEN_SIZE) * info.scale)
                  .set("y", (center.y - 2.0 * TOKEN_SIZE) * info.scale)
                  .set("width", TOKEN_SIZE * info.scale * 4.0)
                  .set("height", TOKEN_SIZE * info.scale * 4.0)
                  .set("rx", TOKEN_SIZE * info.scale)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
                .add(draw_city_circle(&pos4, info))
        }
        x => {
            println!("A tile has an unknown number of circles: {}", x);
            g.add(draw_circle(&pos,
                                       TOKEN_SIZE * info.scale, "red", "none",
                                       0.0))
        }
    }

}

/// Draw the constrast line around a city
pub fn draw_city_contrast(city: tile::City,
                      center: &na::Vector2<f64>,
                      info: &map::MapInfo) -> element::Group {
    let basis = get_basis(&info.orientation);
    let g = element::Group::new();
    let pos = info.scale * (basis * city.position() + center);
    match city.circles {
        1 => {
            let size = (TOKEN_SIZE + LINE_WIDTH) * info.scale;
            g.add(draw_circle(&pos, size, "none", "white",
                                       LINE_WIDTH * info.scale))
        }
        2 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + LINE_WIDTH) * info.scale,
                (TOKEN_SIZE + LINE_WIDTH) * info.scale);
            g.add(element::Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width",
                       (TOKEN_SIZE * 4.0 + LINE_WIDTH * 2.0) * info.scale)
                  .set("height", (TOKEN_SIZE + LINE_WIDTH) * info.scale * 2.0)
                  .set("rx", TOKEN_SIZE * info.scale)
                  .set("stroke", "white")
                  .set("stroke-width", LINE_WIDTH * info.scale)
            )
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = info.scale * TOKEN_SIZE;
            let radius = (TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale;
            let pos1 = pos + na::Vector2::new(0.0, -2.0 * size / sq3);
            let pos2 = pos + na::Vector2::new(-size, size / sq3);
            let pos3 = pos + na::Vector2::new( size, size / sq3);
            let data = Data::new()
                .move_to((-size + pos.x, size / sq3 + size + pos.y))
                .line_to((size + pos.x, size / sq3 + size + pos.y))
                .line_to((size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .line_to((0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .close();
            g.add(element::Path::new()
                    .set("d", data)
                    .set("stroke", "white")
                    .set("stroke-width", 3.0 *LINE_WIDTH * info.scale))
                .add(draw_circle(&pos1, radius, "white", "none", 0.0))
                .add(draw_circle(&pos2, radius, "white", "none", 0.0))
                .add(draw_circle(&pos3, radius, "white", "none", 0.0))
        }
        4 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale,
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale);
            let dim = (4.0 * TOKEN_SIZE + 3.0 * LINE_WIDTH) * info.scale;
            g.add(element::Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width", dim)
                  .set("height", dim)
                  .set("rx", (TOKEN_SIZE + LINE_WIDTH) * info.scale)
                  .set("fill", "white"))
        }
        _ => g,
    }
}

/// Draw a single city circle
pub fn draw_city_circle(pos: &na::Vector2<f64>,
                        info: &map::MapInfo) -> element::Circle {
    draw_circle(pos, TOKEN_SIZE * info.scale, "white", "black",
                LINE_WIDTH * info.scale)
}

/// Draw a stop
pub fn draw_stop<T>(stop: tile::Stop,
                    center: na::Vector2<f64>,
                    info: &map::MapInfo,
                    tile: &T) -> element::Group
    where
        T: tile::TileSpec
{
    let basis = get_basis(&info.orientation);

    let pos = info.scale * (basis * stop.position() + center);
    let a = stop.revenue_angle as f64 * PI / 180.0;
    let text_circle_pos = pos + info.scale *
        na::Matrix2::new(a.cos(), a.sin(), -a.sin(), a.cos()) *
        na::Vector2::new(STOP_TEXT_DIST, 0.0);
    let text_pos = text_circle_pos + info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let text = tile.text(stop.text_id);
    element::Group::new()
        .add(draw_circle(&pos, STOP_SIZE * info.scale, "black",
                                  "white", LINE_WIDTH * info.scale))
        .add(draw_circle(&text_circle_pos,
                                  REVENUE_CIRCLE_RADIUS * info.scale,
                                  "white", "black", LINE_WIDTH * info.scale))
        .add(element::Text::new()
             .add(node::Text::new(text))
             .set("x", text_pos.x)
             .set("y", text_pos.y)
             .set("style", "text-anchor:middle;"))
}

/// Helper to draw circles
pub fn draw_circle(pos: &na::Vector2<f64>, radius: f64, fill: &str,
                   stroke_color: &str, stroke_width: f64) -> element::Circle {
    element::Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", radius)
        .set("fill", fill)
        .set("stroke", stroke_color)
        .set("stroke-width", stroke_width)
}

pub fn get_basis(orientation: &Orientation) -> na::Matrix2x3<f64> {
    match orientation {
        &Orientation::Horizontal => na::Matrix2x3::from_columns(&[
                na::Vector2::new( 1.0,  0.0),
                na::Vector2::new( 0.5, -0.5 * 3.0_f64.sqrt()),
                na::Vector2::new(-0.5, -0.5 * 3.0_f64.sqrt())
            ]),
        &Orientation::Vertical => na::Matrix2x3::from_columns(&[
                na::Vector2::new(0.5 * 3.0_f64.sqrt(), -0.5),
                na::Vector2::new(0.0, -1.0),
                na::Vector2::new(-0.5 * 3.0_f64.sqrt(), -0.5),
            ]),
    }
}

pub fn point_to_tuple(p: na::Vector2<f64>) -> (f64, f64) {
    (p.x, p.y)
}
