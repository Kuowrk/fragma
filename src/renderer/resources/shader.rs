use std::borrow::Cow;
use color_eyre::{eyre::eyre, Result};
use std::path::Path;
use color_eyre::eyre::ErrReport;
#[cfg(target_arch = "wasm32")]
use reqwest::Url;

#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new_from_descriptor(desc: wgpu::ShaderModuleDescriptor, device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(desc);
        Self {
            module
        }
    }

    pub async fn new_from_file(
        filepath: &str,
        device: &wgpu::Device,
    ) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let source = {
            let filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(filepath);
            std::fs::read(filepath)?
        };
        #[cfg(target_arch = "wasm32")]
        let source = fetch_shader_file(filepath).await?;

        let source = match Path::new(filepath)
            .extension()
            .and_then(|ext| ext.to_str()) {
            Some("wgsl") => {
                let source = std::str::from_utf8(&source)?;
                Ok::<wgpu::ShaderSource<'_>, ErrReport>(
                    wgpu::ShaderSource::Wgsl(source.into())
                )
            }
            Some("spv") => Ok::<wgpu::ShaderSource<'_>, ErrReport>(
                wgpu::util::make_spirv(&source)
            ),
            Some(ext) => {
                return Err(eyre!(
                    "Unsupported shader file extension: {}",
                    ext
                ));
            }
            None => {
                return Err(eyre!("No file extension found"));
            }
        }?;

        let desc = wgpu::ShaderModuleDescriptor {
            label: Some(filepath),
            source,
        };
        Ok(Self::new_from_descriptor(desc, device))
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_shader_file(filepath: &str) -> Result<Vec<u8>> {
    let base_url = get_base_url();
    let url = base_url.join(filepath)?;
    log::info!("Fetching shader from: {}", url);
    let response = reqwest::get(url.as_str()).await?;
    Ok(response.bytes().await?.to_vec())
}

#[cfg(target_arch = "wasm32")]
fn get_base_url() -> Url {
    use winit::platform::web::WindowExtWebSys;
    let window = web_sys::window().expect("No window");
    let document = window.document().expect("No document");
    let base_url = if let Ok(Some(base_uri)) = document.base_uri() {
        base_uri
    } else {
        window.location().origin().expect("No origin")
    };
    Url::parse(&base_url)
        .expect(&format!("Failed to parse base URL: {}", base_url))
}