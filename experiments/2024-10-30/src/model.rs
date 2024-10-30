use crate::{export::export, mesh::Mesh};

pub fn model() -> anyhow::Result<()> {
    let vertices = vec![
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [-0.5, 0.5, -0.5],  // 2
        [0.5, 0.5, -0.5],   // 3
        [-0.5, -0.5, 0.5],  // 4
        [0.5, -0.5, 0.5],   // 5
        [-0.5, 0.5, 0.5],   // 6
        [0.5, 0.5, 0.5],    // 7
    ];

    let triangles = vec![
        [0, 4, 6], // left
        [0, 6, 2],
        [1, 3, 7], // right
        [1, 7, 5],
        [0, 1, 5], // front
        [0, 5, 4],
        [2, 7, 3], // back
        [2, 6, 7],
        [0, 2, 1], // bottom
        [1, 2, 3],
        [4, 5, 7], // top
        [4, 7, 6],
    ];

    let mesh = Mesh {
        vertices,
        triangles,
    };

    export(mesh.vertices, mesh.triangles)?;

    Ok(())
}
