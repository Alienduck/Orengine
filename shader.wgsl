// Input from the Vertex Buffer
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

// Get Bind Group 0, Binding 0
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
};

// Group 2 for lighting
@group(2) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) normal: vec3<f32>,
};

// A mat4 takes 4 slots (vec4)
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) world_normal: vec3<f32>,   // Pass normal to fragment
    @location(3) world_position: vec3<f32>, // Pass position to fragment
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Reconstruct the 4x4 matrix from the 4 vectors
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;

    // 1. Calculate world position
    // We assume the model matrix handles rotation/scale/translation
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    // 2. Calculate world normal
    // If we scale the object unevenly, we would need a "Normal Matrix", 
    // but for rotation/translation only, model_matrix is fine.
    // .xyz is important to ignore translation for normals (vectors don't have position)
    out.world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    
    // Order: Projection * View * Model * Position
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    
    return out;
}

// Group 1 = Texture (Defined in Rust code)
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Get base color from texture
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // 2. Ambient light (The minimum light everywhere)
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    // 3. Diffuse light (Directional light)
    let light_dir = normalize(light.position - in.world_position);
    let normal = normalize(in.world_normal);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    // Specular highlight (Shiny spots)
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, normal);

    // Shininess: Higher = smaller, sharper highlight (e.g., 32.0 or 64.0)
    let shininess = 32.0; 
    let specular_strength = 0.5;

    // Calculate the highlight
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular_color = light.color * spec * specular_strength;

    // Combine everything
    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}