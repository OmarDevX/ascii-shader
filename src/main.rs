use std::process::Command;
use std::ffi::CString;
use std::fs;
use std::ptr;
use std::str;
use gl::types::*;
use glfw::Context;

use colored::*;

const WIDTH: usize = 100;
const HEIGHT: usize = 80;


// // Render One Frame
// fn main() {
//     let mut grid: Vec<(String, (u8, u8, u8))> = Vec::new();
//     tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(create_and_modify_image(&mut grid));
//     render(&grid, WIDTH * 2);
// }


fn main() {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::Visible(false));
    let (mut window, _events) = glfw.create_window(
        1,
        1,
        "Invisible Window",
        glfw::WindowMode::Windowed,
    )
    .expect("Failed to create GLFW window.");
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    let mut grid: Vec<(String, (u8, u8, u8))> = vec![];
    
    while !window.should_close() {
        let new_pixels = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(create_and_modify_image(&mut grid));

        // Render the grid
        render(&grid, WIDTH * 2);

        window.swap_buffers();
        glfw.poll_events();
    }
}


async fn create_and_modify_image(grid: &mut Vec<(String, (u8, u8, u8))>) -> Vec<(u8, u8, u8)> {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::Visible(false));
    let (mut window, _events) = glfw
        .create_window(1, 1, "Invisible Window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let compute_shader_src = fs::read_to_string("shader/compute_shader.glsl")
        .expect("Failed to read compute shader file");

    unsafe {
        let compute_shader = gl::CreateShader(gl::COMPUTE_SHADER);
        let c_str_compute_shader = CString::new(compute_shader_src.as_bytes()).unwrap();
        gl::ShaderSource(compute_shader, 1, &c_str_compute_shader.as_ptr(), ptr::null());
        gl::CompileShader(compute_shader);

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

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, compute_shader);
        gl::LinkProgram(shader_program);

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

        gl::BindImageTexture(0, img_output, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA8);
        gl::UseProgram(shader_program);
        gl::DispatchCompute(WIDTH as GLuint, HEIGHT as GLuint, 1);
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        let mut pixels: Vec<u8> = vec![0; WIDTH * HEIGHT * 4];
        gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut std::ffi::c_void,
        );

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
                _ => ".".to_string(),
            };

            grid.push((character.clone(), (r, g, b)));
            grid.push((character, (r, g, b)));
        }

        // Clean up
        gl::DeleteTextures(1, &img_output);
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(compute_shader);
    }

    vec![(0, 0, 0); WIDTH * HEIGHT] // Dummy return
}

fn colorize(character: &str, color: (u8, u8, u8)) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", color.0, color.1, color.2, character)
}


fn render(grid: &Vec<(String, (u8, u8, u8))>, map_size: usize) {
    let mut output = String::new();

    for (index, (character, color)) in grid.iter().enumerate() {
        let colored_char = colorize(character, *color);
        output.push_str(&colored_char);

        if (index + 1) % map_size == 0 {
            output.push('\n');
        }
    }

    print!("{}", output);
}
