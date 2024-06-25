use manganis_cli_support::{AssetManifestExt, ManganisSupportGuard};
use manganis_common::{AssetManifest, Config};
use std::fs;
use std::process::Command;

// use cargo_metadata::Message;

// return the location of the executable generated by cargo
// fn get_executable_location(cargo_output: std::io::BufReader<ChildStdout>) -> PathBuf {
//     let executable = cargo_metadata::Message::parse_stream(cargo_output).find_map(|x| {
//         if let Ok(Message::CompilerArtifact(artifact)) = x {
//             artifact.executable
//         } else {
//             None
//         }
//     });
//     let executable = executable.expect("Failed to find the output binary path. This may happen if you build a library instead of an application");

//     executable.into_std_path_buf()
// }

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    // Debug
    let data = format!("{:?}", args);
    fs::write("./link-args.txt", data).unwrap();

    if Some(&String::from("build")) == args.get(1) {
        println!("BUILD");
        build()
    } else {
        // This must be the linker.

        // This is the location where the assets will be copied to in the filesystem
        let assets_file_location = "./assets";

        // Intercept the linker, getting the paths to the object files.
        let (working_dir, object_files) = manganis_cli_support::linker_intercept(std::env::args());

        // Extract the assets
        let assets = AssetManifest::load(object_files);

        let assets_dir = working_dir.join(working_dir.join(assets_file_location));

        // Remove the old assets
        let _ = std::fs::remove_dir_all(&assets_dir);

        // And copy the static assets to the public directory
        assets.copy_static_assets_to(&assets_dir).unwrap();

        // Then collect the tailwind CSS
        let css = assets.collect_tailwind_css(true, &mut Vec::new());

        // And write the CSS to the public directory
        let tailwind_path = assets_dir.join("tailwind.css");
        std::fs::write(tailwind_path, css).unwrap();
    }
}

fn build() {
    // This is the location where the assets will be served from
    let assets_serve_location = "/assets";

    // First set any settings you need for the build
    Config::default()
        .with_assets_serve_location(assets_serve_location)
        .save();

    // Next, tell manganis that you support assets
    let _guard = ManganisSupportGuard::default();

    // Then build your application
    let current_dir = std::env::current_dir().unwrap();

    let args = ["--release"]; //"--message-format=json-render-diagnostics",
    Command::new("cargo")
        .current_dir(&current_dir)
        .arg("build")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    manganis_cli_support::start_linker_intercept(Some(&current_dir), args).unwrap();
}
