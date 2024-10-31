@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transform * vec4(in.position, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 1.0);
}