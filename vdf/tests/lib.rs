extern crate steamy_vdf as vdf;

#[test]
fn loader() {
    // let config = vdf::load("tests/desktop.vdf").unwrap();
    //
    // assert_eq!(
    //     2.0,
    //     config
    //         .lookup("controller_mappings.version")
    //         .unwrap()
    //         .to::<f32>()
    //         .unwrap()
    // );
    //
    // assert_eq!(
    //     "four_buttons",
    //     config
    //         .lookup("controller_mappings.group.0.mode")
    //         .unwrap()
    //         .as_str()
    //         .unwrap()
    // );
    //
    // assert_eq!(
    //     false,
    //     config
    //         .lookup("controller_mappings.group.1.settings.requires_click")
    //         .unwrap()
    //         .to::<bool>()
    //         .unwrap()
    // );
    //
    // let appmanifest = vdf::load("tests/appmanifest.vdf").unwrap();
    //
    // assert_eq!(
    //     "Team Fortress 2",
    //     appmanifest
    //         .lookup("AppState.installdir")
    //         .unwrap()
    //         .as_str()
    //         .unwrap()
    // );

    let wall = vdf::load("tests/cliff_wall_06.vdf").unwrap();

    assert_eq!(None, wall.lookup("VertexlitGeneric.$detailtexturetransform"));
    assert_eq!(
        "models\\props_forest/cliff_wall_06",
        wall
            .lookup("VertexlitGeneric.$baseTexture")
            .unwrap()
            .as_str()
            .unwrap()
    );
    assert_eq!(
        "1",
        wall
            .lookup("VertexlitGeneric.$seamless_detail")
            .unwrap()
            .as_str()
            .unwrap()
    );
}
