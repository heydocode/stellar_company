use std::fmt::Display;
use reqwest::Client;

use shared::prelude::{JPLHorizonsBodySearch, Mass, Position, Vec3f64, Velocity};

pub async fn get_body_motion(client: Client, id: i64) -> Option<(Position, Velocity)> {
    let url = format!(
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND=%27{}%27&EPHEM_TYPE=VECTORS&CENTER=%27500@399%27&TLIST=%272000-01-01-12-00-00%27&TIME_TYPE=TT&REF_SYSTEM=%27ICRF%27&OUT_UNITS=%27KM-S%27&OBJ_DATA=%27NO%27",
        id
    );
    let text: String = if let Ok(response) = client.get(url).send().await {
        if let Ok(text) = response.text().await {
            text
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.starts_with("$$EOE") {
            break;
        }
        if line.starts_with("$$SOE") {
            // Skip useless line
            lines.next();
            if let Some(should_be_x_line) = lines.next() {
                // Just to make sure everything is good
                if should_be_x_line.contains(" X") {
                    let pos_line = should_be_x_line;
                    if let Some(next_line) = lines.next() {
                        // Another sanity check
                        if next_line.contains("VX") {
                            let vel_line = next_line;
                            if let Some(pos) = parse_position(pos_line) {
                                if let Some(vel) = parse_velocity(vel_line) {
                                    return Some((pos, vel));
                                }
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    None
}

pub async fn search_bodies(client: Client, input: impl Into<String> + Display) -> Option<Vec<JPLHorizonsBodySearch>> {
    let response_buf: String;
    let mut bodies: Vec<JPLHorizonsBodySearch> = Vec::new();
    let search_url = format!(
        // Some experimentation, I don't believe it changes anything
        // "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND='{}'MAKE_EPHEM='NO'",
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND=%27{}%27&MAKE_EPHEM=%27NO%27",
        input
    );
    if let Ok(response) = client.get(search_url).send().await {
        if let Ok(response_buf2) = response.text().await {
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
        if line
            .contains("ID#      Name                               Designation  IAU/aliases/other")
        {
            relevant_lines = true;
            // Skip useless line
            lines.next();
            continue;
        }
        if line.trim().is_empty() && relevant_lines == true {
            break;
        }
        if !relevant_lines {
            continue;
        }
        if relevant_lines {
            if let Some(split) = line.split_at_checked(9) {
                let mut temp_body = JPLHorizonsBodySearch::default();
                let id = split.0.trim().parse::<i64>().ok()?; {
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
                bodies.push(temp_body);
            }
        }
    }
    Some(bodies)
}

/// Returns Mass and radius
pub async fn get_body_properties(client: Client, id: i64) -> Option<(Mass, f64)> {
    let response_buf: String;
    let search_url = format!(
        "https://ssd.jpl.nasa.gov/api/horizons.api?format=text&COMMAND='{}'&MAKE_EPHEM='NO'&OBJ_DATA='YES'",
        id
    );
    if let Ok(response) = client.get(search_url).send().await {
        if let Ok(response_buf2) = response.text().await {
            response_buf = response_buf2;
        } else {
            return None;
        }
    } else {
        return None;
    }

    if let Some(res) = parse_horizons_physical_single(response_buf.as_str()) {
        Some(res)
    } else {
        None
    }
}

fn parse_position(line: &str) -> Option<Position> {
    Some(Position(parse_vec3_line(line)?, Vec3f64::ZERO))
}

fn parse_velocity(line: &str) -> Option<Velocity> {
    Some(Velocity(parse_vec3_line(line)?, Vec3f64::ZERO))
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
    }

    if result.x.is_nan() || result.y.is_nan() || result.z.is_nan() {
        None
    } else {
        Some(result)
    }
}

pub fn parse_horizons_physical_single(text: &str) -> Option<(Mass, f64)> {
    let mut result = (Mass(f64::NAN), f64::NAN);

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains('=') {
            continue;
        }
        
        let segments: Vec<&str> = line.split('=').collect();

        // Iterate through key/value | Pairwise loop
        // TODO: Use it in the symplectic euler physics
        for i in (0..segments.len() - 1).step_by(2) {
            let key_raw = segments[i].trim();
            let value_raw = segments[i + 1].trim();

            let value = if i + 2 < segments.len() {
                let next_key_start = value_raw
                    .find(|c: char| c.is_alphabetic() && !c.is_ascii_digit() && c != '.')
                    .unwrap_or(value_raw.len());
                &value_raw[..next_key_start].trim()
            } else {
                value_raw
            };

            let key = key_raw.to_lowercase();
            
            if key.contains("radius")
                && (key.contains("mean") || key.contains("vol") || key.contains("vol."))
                && !key.contains("equ")
                && !key.contains("polar")
                && !key.contains("solar")
            {
                if let Some(v) = parse_property_number(value) {
                    result.1 = v;
                } else {
                    return None;
                }
            }
            
            if key.contains("mass") {
                let val = value.to_string();
                let mut exp: Option<i32> = None;
                if let Some(pos) = key.rfind("10^") {
                    let exp_str = &key[pos + 3..];
                    let exp_clean = exp_str
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
                        .collect::<String>();
                    if let Ok(e) = exp_clean.parse::<i32>() {
                        exp = Some(e);
                    } else {
                        return None;
                    }
                }
                if let Some(base) = parse_property_number(&val) {
                    if let Some(e) = exp {
                        result.0 = Mass(base * 10f64.powi(e));
                    } else {
                        result.0 = Mass(base);
                    }
                }
            }
        }
    }

    if result.0.0.is_nan() || result.1.is_nan() {
        None
    } else {
        Some(result)
    }
}

fn parse_property_number(s: &str) -> Option<f64> {
    let mut num_str = String::new();
    let mut started = false;

    for c in s.chars() {
        // Third condition is for negative values, that's mainly for reuse,
        // not for mass and radius which can't be negative
        if c.is_ascii_digit() || c == '.' || (c == '-' && num_str.is_empty()) {
            num_str.push(c);
            started = true;
        } 
        // JPL Horizons doesn't use e/E at the moment, but who knows!
        // It's not standardized anyways!
        else if started && matches!(c, 'e' | 'E') {
            num_str.push(c);
        } else if started {
            break;
        }
    }

    if num_str.is_empty() {
        return None;
    }

    num_str.parse::<f64>().ok()
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

/// All tests have been combined due to an inconsistency observed
/// when running a few reqwests in parallel (JPL Horizons starts giving the Home page).
/// 
/// In production, reqwests may even be delayed, to not stress the API, 
/// or the user's internet connection
#[tokio::test]
async fn test_network_retrieval() {
    let client = Client::builder()
            .user_agent("curl/7.79.1")
            .build().unwrap();
    
    // -------------------------------------------
    // TEST get_body_motion
    // -------------------------------------------
    
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
    let testable = get_body_motion(client.clone(), 499).await.unwrap();

    let condition1 = testable.0.0.x.floor() == truth_result.0.0.x.floor()
        && testable.0.0.y.floor() == truth_result.0.0.y.floor()
        && testable.0.0.z.floor() == truth_result.0.0.z.floor();

    let condition2 = testable.1.0.x.floor() == truth_result.1.0.x.floor()
        && testable.1.0.y.floor() == truth_result.1.0.y.floor()
        && testable.1.0.z.floor() == truth_result.1.0.z.floor();

    // It will be only visible if the test fails.
    println!("{}", condition1);
    println!("{} {}", testable.0.0.x.floor(), truth_result.0.0.x.floor());

    assert!(condition1);
    assert!(condition2);
    
    // -------------------------------------------
    // TEST get_body_properties
    // -------------------------------------------
    
    let prop = get_body_properties(client.clone(), 299).await.unwrap();
    assert_eq!(prop.0.0.floor(), 48.685e23f64.floor());

    assert_eq!(prop.1.floor(), 6051.84f64.floor());
    
    // -------------------------------------------
    // TEST search_bodies
    // -------------------------------------------
    
    let suggested_bodies = search_bodies(client, "Mars".to_string()).await.unwrap();
    let mut mars_found = false;
    println!("{:?}", &suggested_bodies);
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
fn test_extractors() {
    let extracted = extract_vec3_from_line("                   VX= 3.095693250734420E+01 VY= 3.176535947901246E+01 VZ= 5.221152230112693E-01     ").unwrap();
    let expected: [&str; 3] = [
        "3.095693250734420E+01",
        "3.176535947901246E+01",
        "5.221152230112693E-01",
    ];
    assert_eq!(extracted, expected);
}

#[test]
fn test_parsers() {
    // -------------------------------------------
    // TEST parse_position
    // -------------------------------------------
    
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
    
    // -------------------------------------------
    // TEST parse_velocity
    // -------------------------------------------

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
    
    // -------------------------------------------
    // TEST parse_horizons_physical_single
    // -------------------------------------------
    
    let input = r#"API VERSION: 1.2
    API SOURCE: NASA/JPL Horizons API

    *******************************************************************************
     Revised: April 12, 2021             Mercury                            199 / 1

     PHYSICAL DATA (updated 2024-Mar-04):
      Vol. Mean Radius (km) =  2439.4+-0.1    Density (g cm^-3)     = 5.427
      Mass x10^23 (kg)      =     3.302       Volume (x10^10 km^3)  = 6.085
      Sidereal rot. period  =    58.6463 d    Sid. rot. rate (rad/s)= 0.00000124001
      Mean solar day        =   175.9421 d    Core radius (km)      = ~1600
      Geometric Albedo      =     0.106       Surface emissivity    = 0.77+-0.06
      GM (km^3/s^2)         = 22031.86855     Equatorial radius, Re = 2440.53 km
      GM 1-sigma (km^3/s^2) =                 Mass ratio (Sun/plnt) = 6023682
      Mom. of Inertia       =     0.33        Equ. gravity  m/s^2   = 3.701
      Atmos. pressure (bar) = < 5x10^-15      Max. angular diam.    = 11.0"
      Mean Temperature (K)  = 440             Visual mag. V(1,0)    = -0.42
      Obliquity to orbit[1] =  2.11' +/- 0.1' Hill's sphere rad. Rp = 94.4
      Sidereal orb. per.    =  0.2408467 y    Mean Orbit vel.  km/s = 47.362
      Sidereal orb. per.    = 87.969257  d    Escape vel. km/s      =  4.435
                                     Perihelion  Aphelion    Mean
      Solar Constant (W/m^2)         14462       6278        9126
      Maximum Planetary IR (W/m^2)   12700       5500        8000
      Minimum Planetary IR (W/m^2)   6           6           6
    *******************************************************************************"#;

    let data = parse_horizons_physical_single(input).unwrap();
    assert_eq!(data.0.0.floor(), 3.302e23f64.floor());
    assert_eq!(data.1.floor(), 2439.4f64.floor());
    
    let input = r#"API VERSION: 1.2
    API SOURCE: NASA/JPL Horizons API

    *******************************************************************************
     Revised: April 12, 2021                Venus                           299 / 2

     PHYSICAL DATA (updated 2020-Oct-19):
      Vol. Mean Radius (km) =  6051.84+-0.01 Density (g/cm^3)      =  5.204
      Mass x10^23 (kg)      =    48.685      Volume (x10^10 km^3)  = 92.843
      Sidereal rot. period  =   243.018484 d Sid. Rot. Rate (rad/s)= -0.00000029924
      Mean solar day        =   116.7490 d   Equ. gravity  m/s^2   =  8.870
      Mom. of Inertia       =     0.33       Core radius (km)      = ~3200
      Geometric Albedo      =     0.65       Potential Love # k2   = ~0.25
      GM (km^3/s^2)         = 324858.592     Equatorial Radius, Re = 6051.893 km
      GM 1-sigma (km^3/s^2) =    +-0.006     Mass ratio (Sun/Venus)= 408523.72
      Atmos. pressure (bar) =  90            Max. angular diam.    =   60.2"
      Mean Temperature (K)  = 735            Visual mag. V(1,0)    =   -4.40
      Obliquity to orbit    = 177.3 deg      Hill's sphere rad.,Rp =  167.1
      Sidereal orb. per., y =   0.61519726   Orbit speed, km/s     =   35.021
      Sidereal orb. per., d = 224.70079922   Escape speed, km/s    =   10.361
                                     Perihelion  Aphelion    Mean
      Solar Constant (W/m^2)         2759         2614       2650
      Maximum Planetary IR (W/m^2)    153         153         153
      Minimum Planetary IR (W/m^2)    153         153         153
    *******************************************************************************"#;

    let data = parse_horizons_physical_single(input).unwrap();
    assert_eq!(data.1, 6051.84);
    assert_eq!(data.0.0, 48.685e23);
    
    // -------------------------------------------
    // TEST parse_property_number
    // -------------------------------------------
    assert_eq!(parse_property_number("1234"), Some(1234.0));
    assert_eq!(parse_property_number("1234e65"), Some(1234e65));
    assert_eq!(parse_property_number("-1234"), Some(-1234.0));
    assert_eq!(parse_property_number("1234.1.2"), None);
}

// Earth has special formatting, which should be hardcoded by
// the `parse_horizons_physical_single` function
// #[test]
// fn test_earth() {
//     let input = r#"API VERSION: 1.2
//     API SOURCE: NASA/JPL Horizons API

//     *******************************************************************************
//      Revised: April 12, 2021                 Earth                              399

//      GEOPHYSICAL PROPERTIES (revised May 9, 2022):
//       Vol. Mean Radius (km)    = 6371.01+-0.02   Mass x10^24 (kg)= 5.97219+-0.0006
//       Equ. radius, km          = 6378.137        Mass layers:
//       Polar axis, km           = 6356.752          Atmos         = 5.1   x 10^18 kg
//       Flattening               = 1/298.257223563   oceans        = 1.4   x 10^21 kg
//       Density, g/cm^3          = 5.51              crust         = 2.6   x 10^22 kg
//       J2 (IERS 2010)           = 0.00108262545     mantle        = 4.043 x 10^24 kg
//       g_p, m/s^2  (polar)      = 9.8321863685      outer core    = 1.835 x 10^24 kg
//       g_e, m/s^2  (equatorial) = 9.7803267715      inner core    = 9.675 x 10^22 kg
//       g_o, m/s^2               = 9.82022         Fluid core rad  = 3480 km
//       GM, km^3/s^2             = 398600.435436   Inner core rad  = 1215 km
//       GM 1-sigma, km^3/s^2     =      0.0014     Escape velocity = 11.186 km/s
//       Rot. Rate (rad/s)        = 0.00007292115   Surface area:
//       Mean sidereal day, hr    = 23.9344695944     land          = 1.48 x 10^8 km
//       Mean solar day 2000.0, s = 86400.002         sea           = 3.62 x 10^8 km
//       Mean solar day 1820.0, s = 86400.0         Love no., k2    = 0.299
//       Moment of inertia        = 0.3308          Atm. pressure   = 1.0 bar
//       Mean surface temp (Ts), K= 287.6           Volume, km^3    = 1.08321 x 10^12
//       Mean effect. temp (Te), K= 255             Magnetic moment = 0.61 gauss Rp^3
//       Geometric albedo         = 0.367           Vis. mag. V(1,0)= -3.86
//       Solar Constant (W/m^2)   = 1367.6 (mean), 1414 (perihelion), 1322 (aphelion)
//      HELIOCENTRIC ORBIT CHARACTERISTICS:
//       Obliquity to orbit, deg  = 23.4392911  Sidereal orb period  = 1.0000174 y
//       Orbital speed, km/s      = 29.79       Sidereal orb period  = 365.25636 d
//       Mean daily motion, deg/d = 0.9856474   Hill's sphere radius = 234.9
//     *******************************************************************************"#;

//     let data = parse_horizons_physical_single(input).unwrap();

//     assert_eq!(data.1, 6371.01);
//     assert_eq!(data.0.0, 6.4171e23);
// }