struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) colour: vec4<u32>,
	@location(3) rounding: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	 @location(0) colour: vec4<f32>,
};

@vertex
fn vs_main(
	in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
	 out.clip_position = vec4<f32>(in.position*2.0 - 1.0, 0.0, 1.0);
	 out.colour = vec4<f32>(in.colour)/255.0;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.colour;
}
