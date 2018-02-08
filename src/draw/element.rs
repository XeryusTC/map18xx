extern crate nalgebra as na;
extern crate svg;

use std::f64::consts::PI;
use std::process;
use ::svg::node;
use ::svg::node::element;
use ::svg::node::element::path::Data;
use self::na::{Vector2, Vector3};

use draw::consts::*;
use draw::helpers::*;
use game;
use game::Orientation;
use tile;
use tile::TextAnchor;

/// Draw accessibility arrows (for red-offboards)
pub fn draw_arrow(arrow: &tile::Coordinate,
                  center: &Vector2<f64>,
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

/// Draw impassable barriers
pub fn draw_barrier(barrier: &game::Barrier,
                    pos: &Vector2<f64>,
                    map: &game::Map) -> element::Line {
    let basis = get_basis(&map.orientation);
    let points = [
        Vector3::new( 0.0,  0.0,  1.0),
        Vector3::new( 0.0,  1.0,  0.0),
        Vector3::new( 1.0,  0.0,  0.0),
        Vector3::new( 0.0,  0.0, -1.0),
        Vector3::new( 0.0, -1.0,  0.0),
        Vector3::new(-1.0,  0.0,  0.0),
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

/// Draw a city
pub fn draw_city(city: tile::City,
                    center: Vector2<f64>,
                    info: &game::Map,
                    tile: &tile::TileSpec,
                    rotation: &f64) -> element::Group
{
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);

    let text_pos = scale(&info) *
        (rot * basis * city.revenue_position() + center);
    let text = tile.get_text(&city.text_id);
    let master = if text.is_empty() {
        element::Group::new()
    } else {
        element::Group::new()
            .add(draw_circle(&text_pos, REVENUE_CIRCLE_RADIUS * scale(&info),
                             "white", "black", LINE_WIDTH * scale(&info)))
            .add(draw_text(&text, &text_pos, &TextAnchor::Middle, None, None))
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
                      center: &Vector2<f64>,
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
                    size, "none", "white", LINE_WIDTH * scale(&info)))
        }
        2 => {
            let center = pos - Vector2::new(
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
            let pos = pos - Vector2::new(
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

/// Draw the coordinate system around a map
pub fn draw_coordinate_system(game: &game::Game,
                              options: &::Options,
                              width: f64,
                              height: f64) -> element::Group {
    let hoffset: f64;
    let voffset: f64 = BORDER+ 1.0;
    let hstride: f64;
    let vstride: f64;
    let mut hnums: u32 = 1;
    let mut vnums: u32 = 1;
    match game.map.orientation {
        Orientation::Horizontal => {
            if !options.debug_coordinates {
                vnums = 2;
            }
            hoffset = BORDER + 1.0;
            hstride = 1.5;
            vstride = 3.0_f64.sqrt() / vnums as f64;
        }
        Orientation::Vertical => {
            if !options.debug_coordinates {
                hnums = 2;
            }
            hoffset = BORDER + 0.5 * 3.0_f64.sqrt();
            hstride = 3.0_f64.sqrt() / hnums as f64;
            vstride = 1.5;
        }
    }
    let mut border = element::Group::new()
        .add(element::Rectangle::new()
            .set("x", BORDER * scale(&game.map))
            .set("y", BORDER * scale(&game.map))
            .set("width", width * scale(&game.map))
            .set("height", height * scale(&game.map))
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", LINE_WIDTH * scale(&game.map)));
    for x in 0..(game.map.width * hnums) {
        let text = if options.debug_coordinates {
            x.to_string()
        } else {
            (x + 1).to_string()
        };
        let x1 = Vector2::new(x as f64 * hstride + hoffset,
                              0.5 * BORDER) * scale(&game.map);
        let x2 = Vector2::new(x as f64 * hstride + hoffset,
                              1.5 * BORDER + height) * scale(&game.map);
        border = border
            .add(draw_text(&text, &x1, &TextAnchor::Middle,
                                    Some("16pt"), Some(600)))
            .add(draw_text(&text, &x2, &TextAnchor::Middle,
                                    Some("16pt"), Some(600)));
    }
    for y in 0..(game.map.height * vnums) {
        let text = if options.debug_coordinates {
            y.to_string()
        } else {
            (y + 1).to_string()
        };
        let y1 = Vector2::new(0.5 * BORDER,
                              y as f64 * vstride + voffset) * scale(&game.map);
        let y2 = Vector2::new(1.5 * BORDER + width,
                              y as f64 * vstride + voffset) * scale(&game.map);
        border = border
            .add(draw_text(&text, &y1, &TextAnchor::Middle,
                                    Some("16pt"), Some(600)))
            .add(draw_text(&text, &y2, &TextAnchor::Middle,
                                    Some("16pt"), Some(600)));
    }
    border
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
pub fn draw_lawson(center: Vector2<f64>,
                   info: &game::Map) -> element::Circle {
        // Add LINE_WIDTH to compensate for stroke being half in the circle
    draw_circle(&(center * scale(&info)),
                (PATH_WIDTH + LINE_WIDTH) * 0.5 * scale(&info),
                "black", "white", LINE_WIDTH * scale(&info))
}

/// Draws the black inside line of a path
pub fn draw_path(path: &tile::Path,
             center: &Vector2<f64>,
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

/// Draws the white contrast lines around a path
pub fn draw_path_contrast(path: &tile::Path,
                          center: &Vector2<f64>,
                          info: &game::Map,
                          rotation: &f64) -> element::Path {
    draw_path_helper(path, center, info, rotation)
        .set("stroke", "white")
        .set("stroke-width", (PATH_WIDTH + 2.0 * LINE_WIDTH) * scale(&info))
}

/// Draw track that changes revenue with phases
pub fn draw_revenue_track(track: &tile::RevenueTrack,
                          center: &Vector2<f64>,
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
         Vector2::new(blocks / 2.0 * REVENUE_WIDTH, REVENUE_HEIGHT / 2.0));

    // Draw the track
    let textpos = topleft + scale(&map) *
        Vector2::new(REVENUE_WIDTH / 2.0, REVENUE_HEIGHT * 0.5);
    let mut g = element::Group::new()
        .add(element::Rectangle::new()
             .set("x", topleft.x)
             .set("y", topleft.y)
             .set("width", REVENUE_WIDTH * scale(&map))
             .set("height", REVENUE_HEIGHT * scale(&map))
             .set("fill", tile::colors::YELLOW.value()))
        .add(draw_text(&track.yellow.to_string(), &textpos,
                       &TextAnchor::Middle, None, None));
    if let Orientation::Vertical = map.orientation {
        let center = scale(&map) * center;
        g = g.set("transform",
                  format!("rotate(-30 {} {})", center.x, center.y));
    }

    let blocks = [(&track.green,  tile::colors::GREEN),
                  (&track.russet, tile::colors::RUSSET),
                  (&track.grey,   tile::colors::GREY)];
    let offset = scale(&map) * Vector2::new(REVENUE_WIDTH, 0.0);
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
                               &TextAnchor::Middle, None, None));
            i += 1.0;
        }
    }

    g
}

/// Draw a stop
pub fn draw_stop(stop: tile::Stop,
                    center: Vector2<f64>,
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
        let text_pos = pos + scale(&info) * rotate(&angle) *
            Vector2::new(STOP_TEXT_DIST, 0.0);
        g = g.add(draw_circle(&text_pos,
                         REVENUE_CIRCLE_RADIUS * scale(&info),
                         "white", "black", LINE_WIDTH * scale(&info)))
            .add(draw_text(&text, &text_pos, &TextAnchor::Middle, None, None));
    }
    g
}

/// Draw text
pub fn draw_text(text: &str,
                 pos: &Vector2<f64>,
                 anchor: &TextAnchor,
                 size: Option<&str>,
                 weight: Option<u32>) -> element::Text {
    let mut style = String::new();
    style.push_str(match anchor {
        &TextAnchor::Start => "text-anchor:start;",
        &TextAnchor::Middle =>"text-anchor:middle;",
        &TextAnchor::End => "text-anchor:end;",
    });
    match size {
        Some(size) => style += &format!("font-size:{};", size),
        None => style.push_str("font-size:80%"),
    };
    if let Some(weight) = weight {
        style.push_str(format!("font-weight:{};", weight).as_str());
    }
    element::Text::new()
        .add(node::Text::new(text.replace("&", "&amp;")))
        .set("x", pos.x)
        .set("y", pos.y)
        .set("style", style)
        .set("dominant-baseline", "middle")
}

/// Draw the token of a company
pub fn draw_token(name: &str,
                  color: &str,
                  is_home: bool,
                  pos: &Vector2<f64>,
                  map: &game::Map) -> element::Group {
    let g = element::Group::new();
    if is_home {
        g.add(draw_circle(
                pos, (TOKEN_SIZE - 2.0_f64.sqrt() * LINE_WIDTH) * scale(map),
                "white", color, 2.0 * LINE_WIDTH * scale(map)))
            .add(draw_text(name, &pos, &TextAnchor::Middle, None, None)
                 .set("fill", color))
    } else {
        g.add(draw_circle(pos, (TOKEN_SIZE - 0.4 * LINE_WIDTH) * scale(map),
                          color, "", 0.0))
            .add(draw_text(name, &pos, &TextAnchor::Middle, None,
                           Some(700))
                 .set("fill", "white"))
    }
}
