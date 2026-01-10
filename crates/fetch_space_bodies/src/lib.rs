use std::fmt::Display;

use shared::prelude::{JPLHorizonsBodySearch, Mass, Position, Vec3f64, Velocity};

pub fn get_body_motion(id: i64) -> Option<(Position, Velocity)> {
    let url = format!(
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND={}&EPHEM_TYPE=VECTORS&CENTER='500@399'&TLIST='2000-01-01-12-00-00'&TIME_TYPE=TT&REF_SYSTEM='ICRF'&OUT_UNITS='KM-S'&OBJ_DATA='NO'",
        id.to_string()
    );
    let text: String = if let Ok(response) = reqwest::blocking::get(url) {
        if let Ok(text) = response.text() {
            text
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.contains("X =") {
            let pos_line = line;
            if let Some(next_line) = lines.peek() {
                if next_line.contains("VX=") {
                    let vel_line = next_line;
                    if let Some(pos) = parse_position(pos_line) {
                        if let Some(vel) = parse_velocity(vel_line) {
                            return Some((pos, vel));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    None
}

pub fn search_bodies(input: impl Into<String> + Display) -> Option<Vec<JPLHorizonsBodySearch>> {
    let response_buf: String;
    let mut bodies: Vec<JPLHorizonsBodySearch> = Vec::new();
    let search_url = format!(
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND='{}'MAKE_EPHEM='NO'",
        input
    );
    if let Ok(response) = reqwest::blocking::get(search_url) {
        if let Ok(response_buf2) = response.text() {
            response_buf = response_buf2;
        } else {
            return None;
        }
    } else {
        return None;
    }

    let mut lines = response_buf.lines();

    let mut relevant_lines = false;
    while let Some(line) = lines.next() {
        let mut temp_body = JPLHorizonsBodySearch::default();
        if line
            .contains("ID#      Name                               Designation  IAU/aliases/other")
        {
            relevant_lines = true;
            lines.next();
            continue;
        }
        if line.trim().is_empty() && relevant_lines == true {
            break;
        }
        if !relevant_lines {
            continue;
        }
        if let Some(split) = line.split_at_checked(9) {
            if let Ok(id) = split.0.trim().parse::<i64>() {
                temp_body.id = id;
            }
            if let Some(split) = split.1.split_at_checked(32 + 2 + 3) {
                let mut name = split.0.to_string();
                if let Some(last_char) = name.chars().last() {
                    if !last_char.is_whitespace() {
                        name += "...";
                    }
                }
                name = name.trim().to_string();
                temp_body.name = name;
                if let Some(split) = split.1.split_at_checked(11 + 1) {
                    let designation = split.0.trim();
                    temp_body.designation = designation.to_string();
                    if let Some(split) = split.1.split_at_checked(19 + 2) {
                        let other = split.0.trim();
                        temp_body.other = other.to_string();
                    }
                }
            }
        }
        bodies.push(temp_body);
    }
    Some(bodies)
}

/// Returns Mass and radius
pub fn get_body_properties(id: i64) -> (Mass, f64) {
    todo!("Using JPL Horizons, retrieve Mass and mean radius")
}
fn parse_position(line: &str) -> Option<Position> {
    let pos: Position;

    if let Some(position_vec) = parse_vec3_line(line) {
        pos = Position(position_vec, Vec3f64::ZERO);
    } else {
        return None;
    }

    Some(pos)
}

fn parse_velocity(line: &str) -> Option<Velocity> {
    let vel: Velocity;

    if let Some(position_vec) = parse_vec3_line(line) {
        vel = Velocity(position_vec, Vec3f64::ZERO);
    } else {
        return None;
    }

    Some(vel)
}

fn parse_vec3_line(line: &str) -> Option<Vec3f64> {
    let mut result = Vec3f64::new(f64::NAN, f64::NAN, f64::NAN);

    if let Some(cords) = extract_vec3_from_line(line) {
        for (index, cord) in cords.iter().enumerate() {
            if let Ok(parsed_cord) = cord.parse::<f64>() {
                let indexed_cord = match index {
                    0 => &mut result.x,
                    1 => &mut result.y,
                    2 => &mut result.z,
                    _ => unreachable!("There is only 3 elements in the array!"),
                };
                *indexed_cord = parsed_cord;
            }
        }
    } else {
        println!("Unable to extract vec3!");
    }

    if result.x.is_nan() || result.y.is_nan() || result.z.is_nan() {
        None
    } else {
        Some(result)
    }
}

fn extract_vec3_from_line(line: &str) -> Option<[&str; 3]> {
    let mut response = [""; 3];
    let mut split_pattern = line.trim().split("=");
    if let Some(x_raw) = split_pattern.nth(1) {
        if let Some(x_number) = x_raw.trim().split_whitespace().next() {
            response[0] = x_number;
        }
    }
    if let Some(y_raw) = split_pattern.nth(0) {
        if let Some(y_number) = y_raw.trim().split_whitespace().next() {
            response[1] = y_number;
        }
    }
    if let Some(z_raw) = split_pattern.nth(0) {
        if let Some(z_number) = z_raw.trim().split_whitespace().next() {
            response[2] = z_number;
        }
    }
    for cord in response {
        if cord.is_empty() {
            return None;
        }
    }
    Some(response)
}

fn extract_n_value(text: &str) -> Option<f64> {
    let n: f64;
    if text.contains('=') {
        let s = text.trim();
        let after_eq = s.split_once('=')?.1.trim();
        if let Ok(val) = after_eq.parse::<f64>() {
            n = val
        } else {
            return None;
        }
    } else {
        if let Ok(val) = text.parse::<f64>() {
            n = val;
        } else {
            return None;
        }
    }
    Some(n)
}

#[test]
fn test_search_bodies() {
    let suggested_bodies = search_bodies("Mars").unwrap();
    let mut mars_found = false;
    println!("{:?}", suggested_bodies);
    for body in suggested_bodies.iter() {
        if body.name == String::from("Mars") {
            mars_found = true;
            assert_eq!(499, body.id);
        }
    }

    if !mars_found {
        panic!("Mars should have been suggested, as it is a full match!");
    }

    // A match that is so close that JPL Horizons is very likely to include it, extremely likely.
    assert!(suggested_bodies.contains(&JPLHorizonsBodySearch {
        id: 4,
        name: "Mars Barycenter".to_string(),
        designation: "".to_string(),
        other: "".to_string()
    }));
}

#[test]
fn test_get_bodies_motion() {
    let truth_result = (
        Position(
            Vec3f64::new(
                2.345471743170112E+08,
                -1.467043494230836E+08,
                -5.155677809885457E+06,
            ),
            Vec3f64::ZERO,
        ),
        Velocity(
            Vec3f64::new(
                3.095693250734420E+01,
                3.176535947901246E+01,
                5.221152230112693E-01,
            ),
            Vec3f64::ZERO,
        ),
    );
    let testable = get_body_motion(499).unwrap();

    let condition1 = testable.0.0.x.floor() == truth_result.0.0.x.floor()
        && testable.0.0.y.floor() == truth_result.0.0.y.floor()
        && testable.0.0.z.floor() == truth_result.0.0.z.floor();

    let condition2 = testable.1.0.x.floor() == truth_result.1.0.x.floor()
        && testable.1.0.y.floor() == truth_result.1.0.y.floor()
        && testable.1.0.z.floor() == truth_result.1.0.z.floor();

    println!("{}", condition1);
    println!("{} {}", testable.0.0.x.floor(), truth_result.0.0.x.floor());

    assert!(condition1);
    assert!(condition2);
}

#[test]
fn test_extractors() {
    let x_text = " X = 2.345471743170112E+08 ";
    let y_text = " Y =-1.467043494230836E+08 ";
    let z_text = "  Z =-5.155677809885457E+06      ";

    let x = extract_n_value(x_text).unwrap();
    let y = extract_n_value(y_text).unwrap();
    let z = extract_n_value(z_text).unwrap();

    assert_eq!(x, 2.345471743170112E+08f64);
    assert_eq!(y, -1.467043494230836E+08f64);
    assert_eq!(z, -5.155677809885457E+06f64);

    let vel_x_text = " VX= 3.095693250734420E+01   ";
    let vel_y_text = " VY= 3.176535947901246E+01 ";
    let vel_z_text = " VZ= 5.221152230112693E-01         ";

    let vel_x = extract_n_value(vel_x_text).unwrap();
    let vel_y = extract_n_value(vel_y_text).unwrap();
    let vel_z = extract_n_value(vel_z_text).unwrap();

    assert_eq!(vel_x, 3.095693250734420E+01f64);
    assert_eq!(vel_y, 3.176535947901246E+01f64);
    assert_eq!(vel_z, 5.221152230112693E-01f64);
}

/// NOTE: This test will always fail if extractors fail
#[test]
fn test_parsers() {
    let pos_text = " X = 2.345471743170112E+08 Y =-1.467043494230836E+08 Z =-5.155677809885457E+06";
    let pos = parse_position(pos_text).unwrap();
    assert_eq!(
        pos,
        Position(
            Vec3f64::new(
                2.345471743170112E+08f64,
                -1.467043494230836E+08f64,
                -5.155677809885457E+06f64
            ),
            Vec3f64::ZERO
        )
    );

    let vel_text =
        "  VX= 3.095693250734420E+01 VY= 3.176535947901246E+01 VZ= 5.221152230112693E-01       ";
    let vel = parse_velocity(vel_text).unwrap();
    assert_eq!(
        vel,
        Velocity(
            Vec3f64::new(
                3.095693250734420E+01,
                3.176535947901246E+01,
                5.221152230112693E-01
            ),
            Vec3f64::ZERO
        )
    );
}
