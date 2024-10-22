use super::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_count: u32,
    pub index_count: u32,
}