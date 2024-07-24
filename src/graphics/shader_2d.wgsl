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
	@location(3) rounding_box: vec4<f32>,
	@location(4) rounding_values: vec4<f32>,
	@location(5) tex: u32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) colour: vec4<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) tex: u32,
	@location(3) rounding_box: vec4<f32>,
	@location(4) rounding_values: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4<f32>(in.position.x*2.0 - 1.0, 1.0 - in.position.y*2.0, 0.0, 1.0);
	out.colour = in.colour;
	out.uv = in.uv;
	out.tex = in.tex;
	out.rounding_box = in.rounding_box;
	out.rounding_values = in.rounding_values/2.0;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	if
		in.uv.x < in.rounding_box.x ||
		in.uv.y < in.rounding_box.y ||
		in.uv.x > in.rounding_box.x + in.rounding_box.z ||
		in.uv.y > in.rounding_box.y + in.rounding_box.w
	{
		discard;
	}
	var scaled_uv = (in.uv - in.rounding_box.xy) / in.rounding_box.zw;
	var quadrant = scaled_uv > vec2<f32>(0.5, 0.5);
	var radius = select(
		select(in.rounding_values.x, in.rounding_values.y, quadrant.x),
		select(in.rounding_values.z, in.rounding_values.w, quadrant.x),
		quadrant.y,
	);
	var scaled_radius = vec2<f32>(radius);//in.rounding_box.zw;
	var center = in.rounding_box.xy + vec2<f32>(
		select(scaled_radius.x, in.rounding_box.z-scaled_radius.x, quadrant.x),
		select(scaled_radius.y, in.rounding_box.w-scaled_radius.y, quadrant.y),
	);
	if
		select(center.x-in.uv.x, in.uv.x-center.x, quadrant.x) > 0.0 &&
		select(center.y-in.uv.y, in.uv.y-center.y, !quadrant.y) < 0.0 &&
		length(in.uv-center) > radius {
		discard;
	}
	/*
	if in.circle != 0 && length(in.uv-vec2<f32>(0.5, 0.5)) > 0.5 {
		discard;
	}
	*/
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
