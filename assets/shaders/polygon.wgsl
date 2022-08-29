#import bevy_pbr::mesh_view_bindings

// This uniform is not needed here, as color attributes are being sent.
// But in the future, the uniform will be used to highlight selected 
// fill meshes
struct PolygonMaterial {
    color: vec4<f32>, 
    show_com: f32,
    selected: f32,
    is_intersecting: f32,
};

@group(1) @binding(0)
var<uniform> uni: PolygonMaterial;

// // Converts a color from sRGB gamma to linear light gamma
fn toLinear(sRGB: vec4<f32>) -> vec4<f32>
{
    let cutoff = vec4<f32>(sRGB < vec4<f32>(0.04045));
    let higher = pow((sRGB + vec4<f32>(0.055))/vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = sRGB/vec4<f32>(12.92);
    return mix(higher, lower, cutoff);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {

    let plaid_width = 10.0;
    var color = uni.color;

    if uni.is_intersecting > 0.5 {
        let intersecting_color = vec4<f32>(1.0, 0.0, 0.0, 0.25);
        color = mix(color, intersecting_color, 0.8);
    }

    if uni.selected > 0.5 && position.x % plaid_width < plaid_width / 2.0 {
        let selector_color = vec4<f32>(1.0, 1.0, 0.0, 0.5);
        color = mix(color, selector_color, 0.2);
    }
    
    if  uni.show_com > 0.5 {
        let hover_color = vec4<f32>(0.0, 0.0, 0.0, 0.5);
        color = mix(color, hover_color, 0.2);
        return toLinear(color);
    }

    return toLinear(color);
}