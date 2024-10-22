use super::super::resources::mesh::Mesh;

#[derive(Debug)]
pub struct Model {
    meshes: Vec<Mesh>,
    // Buffers start as None and are created when the model is loaded
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl Model {

}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.meshes
            .iter()
            .zip(other.meshes.iter())
            .all(|(self_mesh, other_mesh)| self_mesh == other_mesh)
    }
}