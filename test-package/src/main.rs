// The assets must be configured with the [CLI](cli-support/examples/cli.rs) before this example can be run.

use manganis_cli_support::AssetManifestExt;
use manganis_common::{AssetManifest, Config};
use std::path::PathBuf;
use std::env;
use test_package_dependency::IMAGE_ASSET;

// this is necessary so that the compiler / the linker
// don't remove the "manganis" section in the executable.
// Some part of this section must be used in the main
#[link_section = "manganis"]
static USELESS_DATA: u8 = 32;

fn main() {
    assert!(USELESS_DATA == 32);
    tracing_subscriber::fmt::init();

    println!("{:?}", test_package_dependency::AVIF_ASSET);

    // This is the location where the assets will be copied to in the filesystem
    let assets_file_location = "./dist/";
    // This is the location where the assets will be served from
    let assets_serve_location = "dist/";

    // First set any settings you need for the build
    Config::default()
        .with_assets_serve_location(assets_serve_location)
        .save();

    let path_of_exe = env::current_exe().unwrap();
    println!("I am {path_of_exe:?}");

    // Then collect the assets
    let manifest = AssetManifest::load(&path_of_exe);

    // Remove the old assets
    let _ = std::fs::remove_dir_all(assets_file_location);

    // And copy the static assets to the public directory
    manifest
        .copy_static_assets_to(assets_file_location)
        .unwrap();

    // Then collect the tailwind CSS
    let css = manifest.collect_tailwind_css(true, &mut Vec::new());

    // And write the CSS to the public directory
    std::fs::write(format!("{}/tailwind.css", assets_file_location), css).unwrap();

    let class = manganis::classes!("p-10");
    assert_eq!(class, "p-10");
    let path = manganis::mg!(file("./test-package-dependency/src/asset.txt"));
    println!("{}", path);
    assert!(path.starts_with("dist/asset"));
    println!("{}", IMAGE_ASSET);
    assert!(IMAGE_ASSET.starts_with("dist/rustacean"));


    // FIXME: the data stored in the link_section of the nested dependencies
    // is not merged in the final executable.
    // I have to figure out why it works for `linkme`
    let path = PathBuf::from(format!("./{IMAGE_ASSET}"));
    println!("{:?}", path);
    println!("contents: {:?}", std::fs::read(path).unwrap());
}
