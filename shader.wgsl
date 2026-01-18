// Structure de sortie du Vertex Shader
// @builtin(position) dit au GPU : "Ceci est la coordonnée finale écran"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// --- Etape 1 : VERTEX SHADER ---
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // On définit 3 points en dur (x, y). 
    // L'écran va de -1.0 (gauche/bas) à 1.0 (droite/haut)
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),   // Haut centre
        vec2<f32>(-0.5, -0.5), // Bas gauche
        vec2<f32>(0.5, -0.5)   // Bas droite
    );

    // On définit une couleur pour chaque point (R, G, B)
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0), // Rouge
        vec3<f32>(0.0, 1.0, 0.0), // Vert
        vec3<f32>(0.0, 0.0, 1.0)  // Bleu
    );

    let x = pos[in_vertex_index].x;
    let y = pos[in_vertex_index].y;

    // Z = 0.0 (profondeur), W = 1.0 (nécessaire pour les maths 3D)
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = colors[in_vertex_index];
    
    return out;
}

// --- Etape 2 : FRAGMENT SHADER ---
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Le GPU a automatiquement interpolé les couleurs entre les points !
    // On renvoie la couleur finale + Alpha (1.0 = opaque)
    return vec4<f32>(in.color, 1.0);
}