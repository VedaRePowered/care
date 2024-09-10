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

@group(0) @binding(8)
var texture_4: texture_2d<f32>;
@group(0) @binding(9)
var sampler_4: sampler;

@group(0) @binding(10)
var texture_5: texture_2d<f32>;
@group(0) @binding(11)
var sampler_5: sampler;

@group(0) @binding(12)
var texture_6: texture_2d<f32>;
@group(0) @binding(13)
var sampler_6: sampler;

@group(0) @binding(14)
var texture_7: texture_2d<f32>;
@group(0) @binding(15)
var sampler_7: sampler;

@group(0) @binding(16)
var texture_8: texture_2d<f32>;
@group(0) @binding(17)
var sampler_8: sampler;

@group(0) @binding(18)
var texture_9: texture_2d<f32>;
@group(0) @binding(19)
var sampler_9: sampler;

@group(0) @binding(20)
var texture_10: texture_2d<f32>;
@group(0) @binding(21)
var sampler_10: sampler;

@group(0) @binding(22)
var texture_11: texture_2d<f32>;
@group(0) @binding(23)
var sampler_11: sampler;

@group(0) @binding(24)
var texture_12: texture_2d<f32>;
@group(0) @binding(25)
var sampler_12: sampler;

@group(0) @binding(26)
var texture_13: texture_2d<f32>;
@group(0) @binding(27)
var sampler_13: sampler;

@group(0) @binding(28)
var texture_14: texture_2d<f32>;
@group(0) @binding(29)
var sampler_14: sampler;

@group(0) @binding(30)
var texture_15: texture_2d<f32>;
@group(0) @binding(31)
var sampler_15: sampler;

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
		case 5u: { out *= textureSample(texture_4, sampler_4, in.uv); }
		case 6u: { out *= textureSample(texture_5, sampler_5, in.uv); }
		case 7u: { out *= textureSample(texture_6, sampler_6, in.uv); }
		case 8u: { out *= textureSample(texture_7, sampler_7, in.uv); }
		case 9u: { out *= textureSample(texture_8, sampler_8, in.uv); }
		case 10u: { out *= textureSample(texture_9, sampler_9, in.uv); }
		case 11u: { out *= textureSample(texture_10, sampler_10, in.uv); }
		case 12u: { out *= textureSample(texture_11, sampler_11, in.uv); }
		case 13u: { out *= textureSample(texture_12, sampler_12, in.uv); }
		case 14u: { out *= textureSample(texture_13, sampler_13, in.uv); }
		case 15u: { out *= textureSample(texture_14, sampler_14, in.uv); }
		case 16u: { out *= textureSample(texture_15, sampler_15, in.uv); }
		default: { }
	}
	return out;
}
