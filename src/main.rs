use std::ffi::CString;
use std::ptr;
use std::str;
use gl::types::*;
use glfw::Context;

use colored::*;

const WIDTH: usize = 100;
const HEIGHT: usize = 80;

fn main() {
    let mut grid: Vec<(String, (u8, u8, u8))> = Vec::new();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(create_and_modify_image(&mut grid));
    render(&grid, WIDTH * 2);
}

async fn create_and_modify_image(grid: &mut Vec<(String, (u8, u8, u8))>) -> Vec<(u8, u8, u8)> {
    // Initialize GLFW and create an invisible window (required to create an OpenGL context)
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::Visible(false));
    let (mut window, _events) = glfw
        .create_window(1, 1, "Invisible Window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // OpenGL Compute Shader setup
    let compute_shader_src = r#"
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
    "#;

    unsafe {
        // Create and compile compute shader
        let compute_shader = gl::CreateShader(gl::COMPUTE_SHADER);
        let c_str_compute_shader = CString::new(compute_shader_src.as_bytes()).unwrap();
        gl::ShaderSource(compute_shader, 1, &c_str_compute_shader.as_ptr(), ptr::null());
        gl::CompileShader(compute_shader);

        // Check for compilation errors
        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(compute_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetShaderiv(compute_shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut error = vec![0; len as usize];
            gl::GetShaderInfoLog(
                compute_shader,
                len,
                ptr::null_mut(),
                error.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Failed to compile compute shader: {}",
                str::from_utf8(&error).unwrap()
            );
        }

        // Create shader program and link
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, compute_shader);
        gl::LinkProgram(shader_program);

        // Check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut len = 0;
            gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);
            let mut error = vec![0; len as usize];
            gl::GetProgramInfoLog(
                shader_program,
                len,
                ptr::null_mut(),
                error.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "Failed to link shader program: {}",
                str::from_utf8(&error).unwrap()
            );
        }

        // Create texture to store the image
        let mut img_output = 0;
        gl::GenTextures(1, &mut img_output);
        gl::BindTexture(gl::TEXTURE_2D, img_output);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as GLint,
            WIDTH as GLsizei,
            HEIGHT as GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        // Bind the texture to image unit 0
        gl::BindImageTexture(0, img_output, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA8);

        // Use the compute shader
        gl::UseProgram(shader_program);

        // Dispatch the compute shader
        gl::DispatchCompute(WIDTH as GLuint, HEIGHT as GLuint, 1);

        // Ensure all writes to the image are finished
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        // Read back the image data
        let mut pixels: Vec<u8> = vec![0; WIDTH * HEIGHT * 4];
        gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut std::ffi::c_void,
        );

        // Convert the image data to (r, g, b) tuples and ASCII characters
        for i in 0..(WIDTH * HEIGHT) {
            let r = pixels[i * 4];
            let g = pixels[i * 4 + 1];
            let b = pixels[i * 4 + 2];
            let combined_value = r as u32 + g as u32 + b as u32;

            let character = match combined_value {
                0..=30 => "i".to_string(),
                31..=60 => "n".to_string(),
                61..=90 => "a".to_string(),
                91..=120 => "b".to_string(),
                121..=150 => "c".to_string(),
                151..=180 => "d".to_string(),
                181..=210 => "e".to_string(),
                211..=240 => "f".to_string(),
                241..=270 => "g".to_string(),
                271..=300 => "h".to_string(),
                301..=330 => "j".to_string(),
                331..=360 => "k".to_string(),
                361..=390 => "l".to_string(),
                391..=420 => "m".to_string(),
                421..=450 => "o".to_string(),
                451..=480 => "p".to_string(),
                481..=510 => "q".to_string(),
                511..=540 => "r".to_string(),
                541..=570 => "s".to_string(),
                571..=600 => "t".to_string(),
                601..=630 => "u".to_string(),
                631..=660 => "v".to_string(),
                661..=690 => "w".to_string(),
                691..=720 => "x".to_string(),
                721..=750 => "y".to_string(),
                751..=765 => "z".to_string(),
                _ => ".".to_string(), // Default case (if needed)
            };

            grid.push((character.clone(), (r, g, b)));
            grid.push((character, (r, g, b))); // Repeat the character to adjust for aspect ratio
        }

        // Clean up
        gl::DeleteTextures(1, &img_output);
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(compute_shader);
    }

    vec![(0, 0, 0); WIDTH * HEIGHT] // Dummy return
}

fn colorize(character: &str, color: (u8, u8, u8)) -> ColoredString {
    match character {
        "a" => character.red(),
        "b" => character.green(),
        "c" => character.blue(),
        "d" => character.yellow(),
        "e" => character.magenta(),
        "f" => character.cyan(),
        "g" => character.red(),
        "h" => character.green(),
        "i" => character.black(),
        "n" => character.white(),
        "j" => character.yellow(),
        "k" => character.magenta(),
        "l" => character.cyan(),
        "m" => character.red(),
        "o" => character.green(),
        "p" => character.blue(),
        "q" => character.yellow(),
        "r" => character.magenta(),
        "s" => character.cyan(),
        "t" => character.red(),
        "u" => character.green(),
        "v" => character.blue(),
        "w" => character.yellow(),
        "x" => character.magenta(),
        "y" => character.cyan(),
        "z" => character.red(),
        _ => character.black(),
    }
    .truecolor(color.0, color.1, color.2)
}

fn render(grid: &Vec<(String, (u8, u8, u8))>, map_size: usize) {
    for (index, (character, color)) in grid.iter().enumerate() {
        let colored_char = colorize(character, *color);
        print!("{}", colored_char);

        // Print a newline after every map_size elements (assuming a 2D grid layout)
        if (index + 1) % map_size == 0 {
            println!();
        }
    }
}
