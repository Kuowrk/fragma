use color_eyre::Result;
use naga::{
    front::wgsl,
    back::spv,
    valid::{Validator, ValidationFlags, Capabilities}
};
use std::{env, fs, path::Path};
use color_eyre::eyre::OptionExt;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=shaders/*");

    let shaders_dir = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("shaders");

    // Iterate through all WGSL files in the shaders directory
    for entry in fs::read_dir(shaders_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("wgsl") {
            log::warn!("Skipping non-WGSL file: {:?}", path);
            continue;
        }

        // Read the WGSL file
        let source = fs::read_to_string(&path)?;

        // Parse the WGSL source to IR
        let module = wgsl::parse_str(&source)?;

        // Validate the IR
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let info = validator.validate(&module)?;

        // Print the validation info
        log::info!("{:?}", info);

        // Generate the SPIR-V binary
        let options = spv::Options::default();
        let spv_binary = spv::write_vec(&module, &info, &options, None)?;

        // Write the SPIR-V binary to a file
        let shader_name = path
            .file_stem()
            .ok_or_eyre("Shader file has no name")?
            .to_str()
            .ok_or_eyre("Shader file name is not valid UTF-8")?;
        let output_filepath = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
            .join("shaders-compiled")
            .join(format!("{}.spv", shader_name));

        fs::create_dir_all(output_filepath.parent().ok_or_eyre("No parent")?)?;
        fs::write(output_filepath, bytemuck::cast_slice(&spv_binary))?;
    }

    Ok(())
}