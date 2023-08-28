#[cfg(feature = "with-mvt")]
use std::{
    env,
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

#[cfg(feature = "with-mvt")]
fn compile_protos() -> Result<(), Box<dyn std::error::Error>> {
    // override the build location, in order to check in the changes to proto files
    env::set_var("OUT_DIR", "src/mvt");

    if !Path::new("src/mvt/vector_tile.rs").exists() {
        prost_build::compile_protos(&["src/mvt/vector_tile.proto"], &["src/mvt/"])?;
        // read file contents to string
        let mut file = OpenOptions::new()
            .read(true)
            .open("src/mvt/vector_tile.rs")?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        // append warning that file was auto-generate
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("src/mvt/vector_tile.rs")?;
        file.write_all("// This file was automatically generated through the build.rs script, and should not be edited.\n\n".as_bytes())?;
        file.write_all(buffer.as_bytes())?;
    }

    // As the proto file is checked in, the build should not fail if the file is not found
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "with-mvt")]
    compile_protos()?;

    Ok(())
}
