use std::sync::atomic::AtomicUsize;
use super::vertex::Vertex;

#[cfg(not(target_arch = "wasm32"))]
static MESH_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

// This is a workaround for the lack of atomic operations in WebAssembly.
#[cfg(target_arch = "wasm32")]
static mut MESH_ID_COUNTER: u32 = 0;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    id: usize,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Option<Vec<u32>>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let id = MESH_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        #[cfg(target_arch = "wasm32")]
        let id = unsafe {
            let id = MESH_ID_COUNTER;
            MESH_ID_COUNTER += 1;
            id
        };

        Self {
            vertices,
            indices,
            id,
        }
    }

    pub fn new_triangle() -> Self {
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0].into(),
                normal: [0.0, 0.0, 1.0].into(),
                color: [1.0, 0.0, 0.0].into(),
                texcoord: [0.0, 0.0].into(),
            },
            Vertex {
                position: [0.5, -0.5, 0.0].into(),
                normal: [0.0, 0.0, 1.0].into(),
                color: [0.0, 1.0, 0.0].into(),
                texcoord: [0.5, 1.0].into(),
            },
            Vertex {
                position: [0.0, 0.5, 0.0].into(),
                normal: [0.0, 0.0, 1.0].into(),
                color: [0.0, 0.0, 1.0].into(),
                texcoord: [1.0, 0.0].into(),
            },
        ];

        let indices = vec![0, 1, 2];

        Self::new(vertices, Some(indices))
    }

    pub fn new_quad() -> Self {
        // Clockwise winding order
        let vertices = vec![
            // Top left triangle
            Vertex {
                position: [1.0, 1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [1.0, 0.0, 0.0].into(),
                texcoord: [0.0, 0.0].into(),
            },
            Vertex {
                position: [-1.0, -1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [0.0, 1.0, 0.0].into(),
                texcoord: [1.0, 0.0].into(),
            },
            Vertex {
                position: [-1.0, 1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [0.0, 0.0, 1.0].into(),
                texcoord: [0.0, 1.0].into(),
            },
            // Bottom right triangle
            Vertex {
                position: [-1.0, -1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [0.0, 1.0, 0.0].into(),
                texcoord: [1.0, 0.0].into(),
            },
            Vertex {
                position: [1.0, 1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [1.0, 0.0, 1.0].into(),
                texcoord: [1.0, 1.0].into(),
            },
            Vertex {
                position: [1.0, -1.0, 0.0].into(),
                normal: [0.0, 1.0, 0.0].into(),
                color: [0.0, 0.0, 1.0].into(),
                texcoord: [0.0, 1.0].into(),
            },
        ];

        let indices = vec![
            0, 1, 2, // Top left triangle
            3, 4, 5, // Bottom right triangle
        ];

        Self::new(vertices, Some(indices))
    }
}

impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}