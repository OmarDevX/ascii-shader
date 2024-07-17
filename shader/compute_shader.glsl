    #version 430

    layout (local_size_x = 1, local_size_y = 1) in;

    layout (binding = 0, rgba8) uniform image2D img_output;

    void main() {
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    vec2 center = vec2(imageSize(img_output)) / 2.0;
    float radius = float(min(imageSize(img_output).x, imageSize(img_output).y)) / 2.0;

    vec3 light_dir = normalize(vec3(1.0, 1.0, 1.0));
    vec3 sphere_color = vec3(255.0 / 255.0, 127.0 / 255.0, 51.0 / 255.0);
    vec2 dist = vec2(pixel_coords) - center;
    float dist_length = length(dist);
    
    vec4 color = vec4(0.0);
    if (dist_length <= radius) {
        float z = sqrt(radius * radius - dist_length * dist_length);
        vec3 normal = normalize(vec3(dist, z));
        float diffuse = max(dot(normal, light_dir), 0.0);
        vec3 shaded_color = sphere_color * diffuse;
        color = vec4(shaded_color, 1.0);
    }
    imageStore(img_output, pixel_coords, color);
}
    