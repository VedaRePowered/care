// TODO: dynamically generate this
// also, try an array of textures (not an array texture)
@group(0) @binding(0)
var texture_0: texture_2d<f32>;
@group(0) @binding(1)
var sampler_0: sampler;

@group(0) @binding(2)
var texture_1: texture_2d<f32>;
@group(0) @binding(3)
var sampler_1: sampler;

@group(0) @binding(4)
var texture_2: texture_2d<f32>;
@group(0) @binding(5)
var sampler_2: sampler;

@group(0) @binding(6)
var texture_3: texture_2d<f32>;
@group(0) @binding(7)
var sampler_3: sampler;

struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) colour: vec4<f32>,
	@location(3) rounding: f32,
	@location(4) tex: u32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) colour: vec4<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) tex: u32,
	@location(3) circle: i32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4<f32>(in.position.x*2.0 - 1.0, 1.0 - in.position.y*2.0, 0.0, 1.0);
	out.colour = in.colour;
	out.uv = in.uv;
	out.tex = in.tex;
	out.circle = i32(in.rounding >= 0.5);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	if in.circle != 0 && length(in.uv-vec2<f32>(0.5, 0.5)) > 0.5 {
		discard;
	}
	var out: vec4<f32> = in.colour;
	switch in.tex {
		case 1u: { out *= textureSample(texture_0, sampler_0, in.uv); }
		case 2u: { out *= textureSample(texture_1, sampler_1, in.uv); }
		case 3u: { out *= textureSample(texture_2, sampler_2, in.uv); }
		case 4u: { out *= textureSample(texture_3, sampler_3, in.uv); }
		default: { }
	}
	return out;
}
