extern crate nalgebra as na;

use std::f64::consts::PI;
use std::process;
use super::svg::node;
use super::svg::node::element;
use super::svg::node::element::path::Data;
use draw::consts::*;
use game;
use game::Orientation;
use tile;

use super::consts::PPCM;

pub enum TextAnchor {
    Start,
    Middle,
    End,
}

/// Draw text
pub fn draw_text(text: &String,
                 pos: &na::Vector2<f64>,
                 anchor: TextAnchor,
                 size: Option<&str>,
                 weight: Option<u32>) -> element::Text {
    let mut style = String::new();
    style.push_str(match anchor {
        TextAnchor::Start => "text-anchor:start;",
        TextAnchor::Middle =>"text-anchor:middle;",
        TextAnchor::End => "text-anchor:end;",
    });
    match size {
        Some(size) => style.push_str(format!("font-size:{};", size).as_str()),
        None => style.push_str("font-size:80%"),
    }
    if let Some(weight) = weight {
        style.push_str(format!("font-weight:{};", weight).as_str());
    }
    element::Text::new()
        .add(node::Text::new(text.clone()))
        .set("x", pos.x)
        .set("y", pos.y)
        .set("style", style)
}

/// Draw a hexagon
pub fn draw_hex(center: na::Vector2<f64>,
                info: &game::Map) -> element::Path {
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
        .move_to(point_to_tuple(PPCM*info.scale* (basis * points[0] + center)))
        .line_to(point_to_tuple(PPCM*info.scale* (basis * points[1] + center)))
        .line_to(point_to_tuple(PPCM*info.scale* (basis * points[2] + center)))
        .line_to(point_to_tuple(PPCM*info.scale* (basis * points[3] + center)))
        .line_to(point_to_tuple(PPCM*info.scale* (basis * points[4] + center)))
        .line_to(point_to_tuple(PPCM*info.scale* (basis * points[5] + center)))
        .close();

    element::Path::new()
        .set("d", data)
}

/// Draws a border around a hex
pub fn draw_hex_edge(center: na::Vector2<f64>,
                     info: &game::Map) -> element::Path {
    draw_hex(center, info)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH * PPCM * info.scale)
}

/// Draws the background (the color) of a hex
pub fn draw_hex_background(center: na::Vector2<f64>,
                       info: &game::Map,
                       color: tile::colors::Color) -> element::Path {
    draw_hex(center, info)
        .set("fill", color.value())
        .set("stroke", "none")
}

/// Draws the black inside line of a path
pub fn draw_path(path: &tile::Path,
             center: &na::Vector2<f64>,
             info: &game::Map,
             rotation: &f64) -> element::Group {
    let mut g = element::Group::new();
    // Draw an outline if the line is a bridge
    if path.is_bridge() {
        g = g.add(draw_path_contrast(path, center, info, rotation));
    }
    g.add(draw_path_helper(path, center, info, rotation)
          .set("stroke", "black")
          .set("stroke-width", PATH_WIDTH * PPCM * info.scale))
}

/// Helper for drawing the paths, does the actual point calculation
pub fn draw_path_helper(path: &tile::Path,
                        center: &na::Vector2<f64>,
                        info: &game::Map,
                        rotation: &f64) -> element::Path {
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);

    // Calculate end points and control points
    let (start_x, start_y) = point_to_tuple(
        PPCM * info.scale * (rot * basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        PPCM * info.scale * (rot * basis * path.end() + center));
    let control1 = C * rot * basis * path.start() + center;
    let control2 = C * rot * basis * path.end() + center;
    let (x1, y1) = point_to_tuple(PPCM * info.scale * control1);
    let (x2, y2) = point_to_tuple(PPCM * info.scale * control2);

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
                          info: &game::Map,
                          rotation: &f64) -> element::Path {
    draw_path_helper(path, center, info, rotation)
        .set("stroke", "white")
        .set("stroke-width",
             (PATH_WIDTH + 2.0 * LINE_WIDTH) *PPCM * info.scale)
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
pub fn draw_lawson(center: na::Vector2<f64>,
                   info: &game::Map) -> element::Circle {
        // Add LINE_WIDTH to compensate for stroke being half in the circle
    draw_circle(&(center * PPCM * info.scale),
                (PATH_WIDTH + LINE_WIDTH) * 0.5 * PPCM * info.scale,
                "black", "white", LINE_WIDTH * PPCM * info.scale)
}

/// Draw a city
pub fn draw_city<T>(city: tile::City,
                    center: na::Vector2<f64>,
                    info: &game::Map,
                    tile: &T) -> element::Group
    where
        T: tile::TileSpec
{
    let basis = get_basis(&info.orientation);

    let text_circle_pos = PPCM * info.scale * (basis * city.revenue_position()
                                        + center);
    let text_pos = text_circle_pos + PPCM * info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let text = tile.text(city.text_id);
    let master = element::Group::new()
        .add(draw_circle(&text_circle_pos,
                         REVENUE_CIRCLE_RADIUS * PPCM * info.scale,
                         "white", "black", LINE_WIDTH * PPCM * info.scale))
        .add(draw_text(&text, &text_pos, TextAnchor::Middle, None, None));

    let pos = PPCM * info.scale * (basis * city.position() + center);
    let center = basis * city.position() + center;
    let mut g = element::Group::new();
    if let Orientation::Vertical = info.orientation {
        g = g.set("transform", format!("rotate(-30 {} {})", pos.x, pos.y));
    }
    g = match city.circles {
        1 => g.add(draw_city_circle(&pos, info)),
        2 => {
            let pos1 = pos + na::Vector2::new(-PPCM * info.scale * TOKEN_SIZE,
                                              0.0);
            let pos2 = pos + na::Vector2::new( PPCM * info.scale * TOKEN_SIZE,
                                               0.0);
            g.add(element::Rectangle::new()
                  .set("x", (center.x - TOKEN_SIZE) * PPCM * info.scale)
                  .set("y", (center.y - TOKEN_SIZE) * PPCM * info.scale)
                  .set("width", TOKEN_SIZE * PPCM * info.scale * 2.0)
                  .set("height", TOKEN_SIZE * PPCM * info.scale * 2.0)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * PPCM * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = PPCM * info.scale * TOKEN_SIZE;
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
                    .set("stroke-width", LINE_WIDTH * PPCM * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
        }
        4 => {
            let size = PPCM * info.scale * TOKEN_SIZE;
            let pos1 = pos + na::Vector2::new(-size, -size);
            let pos2 = pos + na::Vector2::new(-size,  size);
            let pos3 = pos + na::Vector2::new( size, -size);
            let pos4 = pos + na::Vector2::new( size,  size);
            g.add(element::Rectangle::new()
                  .set("x", (center.x - 2.0 * TOKEN_SIZE) * PPCM * info.scale)
                  .set("y", (center.y - 2.0 * TOKEN_SIZE) * PPCM * info.scale)
                  .set("width", TOKEN_SIZE * PPCM * info.scale * 4.0)
                  .set("height", TOKEN_SIZE * PPCM * info.scale * 4.0)
                  .set("rx", TOKEN_SIZE * PPCM * info.scale)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * PPCM * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
                .add(draw_city_circle(&pos4, info))
        }
        x => {
            println!("A tile has an unknown number of circles: {}", x);
            g.add(draw_circle(&pos,
                              TOKEN_SIZE * PPCM * info.scale, "red", "none",
                              0.0))
        }
    };
    master.add(g)
}

/// Draw the constrast line around a city
pub fn draw_city_contrast(city: tile::City,
                      center: &na::Vector2<f64>,
                      info: &game::Map) -> element::Group {
    let basis = get_basis(&info.orientation);
    let mut g = element::Group::new();
    let pos = PPCM * info.scale * (basis * city.position() + center);
    if let Orientation::Vertical = info.orientation {
        g = g.set("transform", format!("rotate(-30 {} {})", pos.x, pos.y));
    }
    match city.circles {
        1 => {
            let size = (TOKEN_SIZE + LINE_WIDTH) * PPCM * info.scale;
            g.add(draw_circle(&pos, size, "none", "white",
                              LINE_WIDTH * PPCM * info.scale))
        }
        2 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + LINE_WIDTH) * PPCM * info.scale,
                (TOKEN_SIZE + LINE_WIDTH) * PPCM * info.scale);
            g.add(element::Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width",
                       (TOKEN_SIZE * 4.0 + LINE_WIDTH * 2.0) *
                        PPCM * info.scale)
                  .set("height",
                       (TOKEN_SIZE + LINE_WIDTH) * PPCM * info.scale * 2.0)
                  .set("rx", TOKEN_SIZE * PPCM * info.scale)
                  .set("stroke", "white")
                  .set("stroke-width", LINE_WIDTH * PPCM * info.scale)
            )
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = PPCM * info.scale * TOKEN_SIZE;
            let radius = (TOKEN_SIZE + 1.5 * LINE_WIDTH) * PPCM * info.scale;
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
                    .set("stroke-width", 3.0 *LINE_WIDTH * PPCM * info.scale))
                .add(draw_circle(&pos1, radius, "white", "none", 0.0))
                .add(draw_circle(&pos2, radius, "white", "none", 0.0))
                .add(draw_circle(&pos3, radius, "white", "none", 0.0))
        }
        4 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * PPCM * info.scale,
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * PPCM * info.scale);
            let dim = (4.0 * TOKEN_SIZE + 3.0 * LINE_WIDTH) * PPCM *info.scale;
            g.add(element::Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width", dim)
                  .set("height", dim)
                  .set("rx", (TOKEN_SIZE + LINE_WIDTH) * PPCM * info.scale)
                  .set("fill", "white"))
        }
        _ => g,
    }
}

/// Draw a single city circle
pub fn draw_city_circle(pos: &na::Vector2<f64>,
                        info: &game::Map) -> element::Circle {
    draw_circle(pos, TOKEN_SIZE * PPCM * info.scale, "white", "black",
                LINE_WIDTH * PPCM * info.scale)
}

/// Draw a stop
pub fn draw_stop<T>(stop: tile::Stop,
                    center: na::Vector2<f64>,
                    info: &game::Map,
                    tile: &T) -> element::Group
    where
        T: tile::TileSpec
{
    let basis = get_basis(&info.orientation);

    let pos = PPCM * info.scale * (basis * stop.position() + center);
    let a = stop.revenue_angle as f64 * PI / 180.0;
    let text_circle_pos = pos + PPCM * info.scale *
        na::Matrix2::new(a.cos(), a.sin(), -a.sin(), a.cos()) *
        na::Vector2::new(STOP_TEXT_DIST, 0.0);
    let text_pos = text_circle_pos + PPCM * info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let text = tile.text(stop.text_id);
    element::Group::new()
        .add(draw_circle(&pos, STOP_SIZE * PPCM * info.scale, "black",
                         "white", LINE_WIDTH * PPCM * info.scale))
        .add(draw_circle(&text_circle_pos,
                         REVENUE_CIRCLE_RADIUS * PPCM * info.scale,
                         "white", "black", LINE_WIDTH * PPCM * info.scale))
        .add(draw_text(&text, &text_pos, TextAnchor::Middle, None, None))
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

pub fn rotate(theta: &f64) -> na::Matrix2<f64> {
    na::Matrix2::new(theta.cos(), -theta.sin(), theta.sin(), theta.cos())
}

pub fn point_to_tuple(p: na::Vector2<f64>) -> (f64, f64) {
    (p.x, p.y)
}

pub fn draw_barrier(barrier: &game::Barrier,
                    pos: &na::Vector2<f64>,
                    map: &game::Map) -> element::Line {
    let basis = get_basis(&map.orientation);
    let points = [
        na::Vector3::new( 0.0,  0.0,  1.0),
        na::Vector3::new( 0.0,  1.0,  0.0),
        na::Vector3::new( 1.0,  0.0,  0.0),
        na::Vector3::new( 0.0,  0.0, -1.0),
        na::Vector3::new( 0.0, -1.0,  0.0),
        na::Vector3::new(-1.0,  0.0,  0.0),
    ];
    let coords = match barrier.side.as_str() {
        "N"  => (points[0], points[1]),
        "NE" => (points[1], points[2]),
        "SE" => (points[2], points[3]),
        "S"  => (points[3], points[4]),
        "SW" => (points[4], points[5]),
        "NW" => (points[5], points[0]),
        s => {
            eprintln!("Unknown tile side {}", s);
            process::exit(1);
        }
    };
    let start = (pos + basis * coords.0) * PPCM * map.scale;
    let end = (pos + basis * coords.1) * PPCM * map.scale;

    element::Line::new()
        .set("x1", start.x)
        .set("y1", start.y)
        .set("x2", end.x)
        .set("y2", end.y)
        .set("stroke", tile::colors::BARRIER.value())
        .set("stroke-width", BARRIER_WIDTH * PPCM * map.scale)
        .set("stroke-linecap", "round")
}
