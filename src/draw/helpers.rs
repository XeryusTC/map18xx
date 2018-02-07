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

/// Calculate scale for a map
pub fn scale(map: &game::Map) -> f64 {
    map.scale * PPCM
}

/// Draw text
pub fn draw_text(text: &str,
                 pos: &na::Vector2<f64>,
                 anchor: &tile::TextAnchor,
                 size: Option<&str>,
                 weight: Option<u32>) -> element::Text {
    let mut style = String::new();
    style.push_str(match anchor {
        &tile::TextAnchor::Start => "text-anchor:start;",
        &tile::TextAnchor::Middle =>"text-anchor:middle;",
        &tile::TextAnchor::End => "text-anchor:end;",
    });
    match size {
        Some(size) => style += &format!("font-size:{};", size),
        None => style.push_str("font-size:80%"),
    };
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
        .move_to(point_to_tuple(scale(&info) * (basis * points[0] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[1] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[2] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[3] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[4] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[5] + center)))
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
        .set("stroke-width", LINE_WIDTH * scale(&info))
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
          .set("stroke-width", PATH_WIDTH * scale(&info)))
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
        scale(&info) * (rot * basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        scale(&info) * (rot * basis * path.end() + center));
    let control1 = match &path.start_control {
        &None => path.radius() * C * rot * basis * path.start() + center,
        &Some(ref point) => rot * basis * point.as_vector() + center,
    };
    let control2 = match &path.end_control {
        &None => path.radius() * C * rot * basis * path.end() + center,
        &Some(ref point) => rot * basis * point.as_vector() + center,
    };
    let (x1, y1) = point_to_tuple(scale(&info) * control1);
    let (x2, y2) = point_to_tuple(scale(&info) * control2);

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
        .set("stroke-width", (PATH_WIDTH + 2.0 * LINE_WIDTH) * scale(&info))
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
pub fn draw_lawson(center: na::Vector2<f64>,
                   info: &game::Map) -> element::Circle {
        // Add LINE_WIDTH to compensate for stroke being half in the circle
    draw_circle(&(center * scale(&info)),
                (PATH_WIDTH + LINE_WIDTH) * 0.5 * scale(&info),
                "black", "white", LINE_WIDTH * scale(&info))
}

/// Draw a city
pub fn draw_city(city: tile::City,
                    center: na::Vector2<f64>,
                    info: &game::Map,
                    tile: &tile::TileSpec,
                    rotation: &f64) -> element::Group
{
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);

    let text_circle_pos = scale(&info) *
        (rot * basis * city.revenue_position() + center);
    let text_pos = text_circle_pos + scale(&info) *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let text = tile.get_text(&city.text_id);
    let master = if text.is_empty() {
        element::Group::new()
    } else {
        element::Group::new()
            .add(draw_circle(&text_circle_pos,
                             REVENUE_CIRCLE_RADIUS * scale(&info),
                             "white", "black", LINE_WIDTH * scale(&info)))
            .add(draw_text(&text, &text_pos,
                           &tile::TextAnchor::Middle, None, None))
    };

    let pos = scale(&info) * (rot * basis * city.position() + center);
    let mut g = element::Group::new();
    if let Orientation::Vertical = info.orientation {
        g = g.set("transform", format!("rotate(-30 {} {})", pos.x, pos.y));
    }
    g = match city.circles {
        1 => g, // Ignore this, the circle is drawn at the end
        2 => {
            let center = rot * basis * city.position() + center;
            g.add(element::Rectangle::new()
                  .set("x", (center.x - TOKEN_SIZE) * scale(&info))
                  .set("y", (center.y - TOKEN_SIZE) * scale(&info))
                  .set("width", TOKEN_SIZE * scale(&info) * 2.0)
                  .set("height", TOKEN_SIZE * scale(&info) * 2.0)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * scale(&info))
                  .set("transform",
                       format!("rotate({} {} {})", rotation / PI * 180.0,
                               pos.x, pos.y)))
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = scale(&info) * TOKEN_SIZE;
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
                    .set("stroke-width", LINE_WIDTH * scale(&info)))
        }
        4 => {
            g.add(element::Rectangle::new()
                  .set("x", (center.x - 2.0 * TOKEN_SIZE) * scale(&info))
                  .set("y", (center.y - 2.0 * TOKEN_SIZE) * scale(&info))
                  .set("width", TOKEN_SIZE * scale(&info) * 4.0)
                  .set("height", TOKEN_SIZE * scale(&info) * 4.0)
                  .set("rx", TOKEN_SIZE * scale(&info))
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * scale(&info)))
        }
        x => {
            println!("A tile has an unknown number of circles: {}", x);
            g.add(draw_circle(&pos, TOKEN_SIZE * scale(&info), "red", "none",
                              0.0))
        }
    };
    for i in 0..city.circles {
        g = g.add(draw_city_circle(&city_circle_pos(&city, i, &center, info,
                                                   rotation), info));
    }
    master.add(g)
}

/// Draw the constrast line around a city
pub fn draw_city_contrast(city: tile::City,
                      center: &na::Vector2<f64>,
                      info: &game::Map,
                      rotation: &f64) -> element::Group {
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);
    let mut g = element::Group::new();
    let pos = scale(&info) * (rot * basis * city.position() + center);
    if let Orientation::Vertical = info.orientation {
        g = g.set("transform", format!("rotate(-30 {} {})", pos.x, pos.y));
    }
    match city.circles {
        1 => {
            let size = (TOKEN_SIZE + LINE_WIDTH) * scale(&info);
            g.add(draw_circle(
                    &(city_circle_pos(&city, 0, center, info, rotation)),
                              size, "none", "white",
                              LINE_WIDTH * scale(&info)))
        }
        2 => {
            let center = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + LINE_WIDTH) * scale(&info),
                (TOKEN_SIZE + LINE_WIDTH) * scale(&info));
            g.add(element::Rectangle::new()
                  .set("x", center.x)
                  .set("y", center.y)
                  .set("width",
                       (TOKEN_SIZE * 4.0 + LINE_WIDTH * 2.0) * scale(&info))
                  .set("height",
                       (TOKEN_SIZE + LINE_WIDTH) * scale(&info) * 2.0)
                  .set("rx", TOKEN_SIZE * scale(&info))
                  .set("stroke", "white")
                  .set("stroke-width", LINE_WIDTH * scale(&info))
                  .set("transform",
                       format!("rotate({} {} {})", rotation / PI * 180.0,
                               pos.x, pos.y))
            )
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = scale(&info) * TOKEN_SIZE;
            let radius = (TOKEN_SIZE + 1.5 * LINE_WIDTH) * scale(&info);
            let pos1 = city_circle_pos(&city, 0, center, info, rotation);
            let pos2 = city_circle_pos(&city, 1, center, info, rotation);
            let pos3 = city_circle_pos(&city, 2, center, info, rotation);
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
                    .set("stroke-width", 3.0 *LINE_WIDTH * scale(&info)))
                .add(draw_circle(&pos1, radius, "white", "none", 0.0))
                .add(draw_circle(&pos2, radius, "white", "none", 0.0))
                .add(draw_circle(&pos3, radius, "white", "none", 0.0))
        }
        4 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * scale(&info),
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * scale(&info));
            let dim = (4.0 * TOKEN_SIZE + 3.0 * LINE_WIDTH) * scale(&info);
            g.add(element::Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width", dim)
                  .set("height", dim)
                  .set("rx", (TOKEN_SIZE + LINE_WIDTH) * scale(&info))
                  .set("fill", "white"))
        }
        _ => g,
    }
}

/// Draw a single city circle
pub fn draw_city_circle(pos: &na::Vector2<f64>,
                        info: &game::Map) -> element::Circle {
    draw_circle(pos, TOKEN_SIZE * scale(&info), "white", "black",
                LINE_WIDTH * scale(&info))
}

/// Calculate the position of a single circle in a city
pub fn city_circle_pos(city: &tile::City,
                   circle: u32,
                   center: &na::Vector2<f64>,
                   info: &game::Map,
                   rotation: &f64) -> na::Vector2<f64> {
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);
    let pos = rot * basis * city.position() + center;
    let pos = match city.circles {
        1 => pos,
        2 => match circle {
            0 => pos - rot * na::Vector2::new(TOKEN_SIZE, 0.0),
            1 => pos + rot * na::Vector2::new(TOKEN_SIZE, 0.0),
            n => panic!("Illegal circle id {} for city of size 2", n),
        },
        3 => match circle {
            0 => pos + na::Vector2::new(0.0, -2.0 * TOKEN_SIZE/3.0_f64.sqrt()),
            1 => pos + na::Vector2::new(-TOKEN_SIZE,
                                        TOKEN_SIZE / 3.0_f64.sqrt()),
            2 => pos + na::Vector2::new(TOKEN_SIZE, TOKEN_SIZE/3.0_f64.sqrt()),
            n => panic!("Illegal circle id {} for city of size 3", n),
        },
        4 => match circle {
            0 => pos + na::Vector2::new(-TOKEN_SIZE, -TOKEN_SIZE),
            1 => pos + na::Vector2::new(-TOKEN_SIZE,  TOKEN_SIZE),
            2 => pos + na::Vector2::new( TOKEN_SIZE, -TOKEN_SIZE),
            3 => pos + na::Vector2::new( TOKEN_SIZE,  TOKEN_SIZE),
            n => panic!("Illegal circle id {} for city of size 3", n),
        }
        n => panic!("Cities of {} not supported!", n),
    };
    pos * scale(&info)
}

/// Draw a stop
pub fn draw_stop(stop: tile::Stop,
                    center: na::Vector2<f64>,
                    info: &game::Map,
                    tile: &tile::TileSpec,
                    rotation: &f64) -> element::Group
{
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);

    let pos = scale(&info) * (rot * basis * stop.position() + center);
    let mut angle = stop.revenue_angle as f64 * PI / 180.0 + rotation;
    if let Orientation::Vertical = info.orientation {
        angle -= PI / 6.0;
    }
    // Draw the stop
    let mut g = element::Group::new()
        .add(draw_circle(&pos, STOP_SIZE * scale(&info), "black",
                         "white", LINE_WIDTH * scale(&info)));
    // Draw the revenue if it is set
    let text = tile.get_text(&stop.text_id);
    if !text.is_empty() {
        let text_circle_pos = pos + scale(&info) * rotate(&angle) *
            na::Vector2::new(STOP_TEXT_DIST, 0.0);
        let text_pos = text_circle_pos + scale(&info) *
            na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
        g = g.add(draw_circle(&text_circle_pos,
                         REVENUE_CIRCLE_RADIUS * scale(&info),
                         "white", "black", LINE_WIDTH * scale(&info)))
            .add(draw_text(&text, &text_pos, &tile::TextAnchor::Middle, None,
                           None));
    }
    g
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
    let start = (pos + basis * coords.0) * scale(&map);
    let end = (pos + basis * coords.1) * scale(&map);

    element::Line::new()
        .set("x1", start.x)
        .set("y1", start.y)
        .set("x2", end.x)
        .set("y2", end.y)
        .set("stroke", tile::colors::BARRIER.value())
        .set("stroke-width", BARRIER_WIDTH * scale(&map))
        .set("stroke-linecap", "round")
}

pub fn draw_arrow(arrow: &tile::Coordinate,
                  center: &na::Vector2<f64>,
                  map: &game::Map) -> element::Group {
    let basis = get_basis(&map.orientation);

    // Arrow
    let pos1 = (1.0 - ARROW_SIZE * ARROW_LENGTH) * arrow.as_vector();
    let pos2 = arrow.as_vector() + ARROW_SIZE * arrow.as_vector().map(
        |x| if x != 0.0 { 0.0 } else {0.5});
    let pos3 = arrow.as_vector() - ARROW_SIZE * arrow.as_vector().map(
        |x| if x != 0.0 { 0.0 } else {0.5});
    let path = Data::new()
        .move_to(point_to_tuple(scale(&map) * (basis * pos1 + center)))
        .line_to(point_to_tuple(scale(&map) * (basis * pos2 + center)))
        .line_to(point_to_tuple(scale(&map) * (basis * pos3 + center)))
        .close();

    // Contrast
    let pos1 = (1.0 - ARROW_SIZE * ARROW_LENGTH - 2.0 * LINE_WIDTH) *
        arrow.as_vector();
    let pos2 = arrow.as_vector() + arrow.as_vector().map(
        |x| if x != 0.0 { 0.0 } else {0.5}) * (ARROW_SIZE + 2.0 * LINE_WIDTH);
    let pos3 = arrow.as_vector() - arrow.as_vector().map(
        |x| if x != 0.0 { 0.0 } else {0.5}) * (ARROW_SIZE + 2.0 * LINE_WIDTH);
    let contrast = Data::new()
        .move_to(point_to_tuple(scale(&map) * (basis * pos1 + center)))
        .line_to(point_to_tuple(scale(&map) * (basis * pos2 + center)))
        .line_to(point_to_tuple(scale(&map) * (basis * pos3 + center)))
        .close();


    element::Group::new()
        .add(element::Path::new()
            .set("d", contrast)
            .set("fill", "white"))
        .add(element:: Path::new()
             .set("d", path)
             .set("fill", "black"))
}

pub fn draw_revenue_track(track: &tile::RevenueTrack,
                          center: &na::Vector2<f64>,
                          map: &game::Map) -> element::Group {
    let basis = get_basis(&map.orientation);
    // Determine position
    let mut blocks = 1.0;
    if let Some(_) = track.green {
        blocks += 1.0;
    }
    if let Some(_) = track.russet {
        blocks += 1.0;
    }
    if let Some(_) = track.grey {
        blocks += 1.0;
    }
    let topleft = scale(&map) * ((basis * track.position() + center) -
         na::Vector2::new(blocks / 2.0 * REVENUE_WIDTH, REVENUE_HEIGHT / 2.0));

    // Draw the track
    let textpos = topleft + scale(&map) *
        na::Vector2::new(REVENUE_WIDTH / 2.0, REVENUE_HEIGHT * 0.8);
    let mut g = element::Group::new()
        .add(element::Rectangle::new()
             .set("x", topleft.x)
             .set("y", topleft.y)
             .set("width", REVENUE_WIDTH * scale(&map))
             .set("height", REVENUE_HEIGHT * scale(&map))
             .set("fill", tile::colors::YELLOW.value()))
        .add(draw_text(&track.yellow.to_string(), &textpos,
                       &tile::TextAnchor::Middle, None, None));
    if let Orientation::Vertical = map.orientation {
        let center = scale(&map) * center;
        g = g.set("transform",
                  format!("rotate(-30 {} {})", center.x, center.y));
    }

    let blocks = [(&track.green,  tile::colors::GREEN),
                  (&track.russet, tile::colors::RUSSET),
                  (&track.grey,   tile::colors::GREY)];
    let offset = scale(&map) * na::Vector2::new(REVENUE_WIDTH, 0.0);
    let mut i = 1.0;
    for block in blocks.iter() {
        if let &(&Some(ref text), ref color) = block {
            g = g.add(element::Rectangle::new()
                  .set("x", topleft.x + i * offset.x)
                  .set("y", topleft.y)
                  .set("width", REVENUE_WIDTH * scale(&map))
                  .set("height", REVENUE_HEIGHT * scale(&map))
                  .set("fill", color.value()))
                .add(draw_text(&text.to_string(), &(textpos + i * offset),
                               &tile::TextAnchor::Middle, None, None));
            i += 1.0;
        }
    }

    g
}

/// Draw the token of a company
pub fn draw_token(name: &str,
                  color: &str,
                  pos: &na::Vector2<f64>,
                  map: &game::Map) -> element::Group {
    element::Group::new()
        .add(draw_circle(pos, (TOKEN_SIZE - 1.414 * LINE_WIDTH) * scale(map),
                         "white",
                         color, 2.0 * LINE_WIDTH * scale(map)))
        .add(draw_text(name,
                       &(pos + na::Vector2::new(0.0, TOKEN_SIZE / 3.5 * scale(map))),
                       &tile::TextAnchor::Middle, None, None)
             .set("fill", color))
}
