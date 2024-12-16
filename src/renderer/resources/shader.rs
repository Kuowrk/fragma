use color_eyre::Result;
use std::path::Path;

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

    pub async fn new_from_file(filename: &str, device: &wgpu::Device) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let source = {
            let filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("shaders")
                .join(filename);
            std::fs::read_to_string(filepath)?
        };
        #[cfg(target_arch = "wasm32")]
        let source = fetch_shader_file(filename).await?;
        let desc = wgpu::ShaderModuleDescriptor {
            label: Some(filename),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        };
        Ok(Self::new_from_descriptor(desc, device))
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_shader_file(filename: &str) -> Result<String> {
    let base_url = get_base_url();
    let url = base_url.join(&format!("shaders/{}", filename))?;
    log::info!("Fetching shader from: {}", url);
    let response = reqwest::get(url.as_str()).await?;
    Ok(response.text().await?)
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