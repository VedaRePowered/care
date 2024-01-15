@group(0) @binding(0)
var texture_0: texture_2d<f32>;
@group(0) @binding(1)
var sampler_0: sampler;

struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) colour: vec4<f32>,
	@location(3) rounding: vec2<f32>,
	@location(4) tex: u32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) colour: vec4<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) tex: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4<f32>(in.position.x*2.0 - 1.0, 1.0 - in.position.y*2.0, 0.0, 1.0);
	out.colour = in.colour;
	out.uv = in.uv;
	out.tex = in.tex;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	if (in.tex != u32(0)) {
		return in.colour * textureSample(texture_0, sampler_0, in.uv).wwww;
	} else {
		return in.colour;
	}
}
