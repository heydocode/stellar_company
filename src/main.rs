use bevy::prelude::*;
use solar_company::SolarCompanyGameLib;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(SolarCompanyGameLib);
    app.run()
}
