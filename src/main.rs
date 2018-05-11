use std::mem;
use std::ptr;
use std::f64;
use std::thread;
use std::time::Duration;

extern crate gl;
extern crate glutin;

use glutin::GlContext;

fn now() -> f64 {
    let duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("");
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

static VERTEX_DATA: [f32; 12] = [
    -1.0, -1.0,
    -1.0,  1.0,
     1.0,  1.0,
    -1.0, -1.0,
     1.0, -1.0,
     1.0,  1.0];

const VS_SRC: &'static [u8] = b"
    #version 100
    precision mediump float;
    attribute vec2 vertex_pos;
    varying vec2 screen_pos;
    void main() {
        gl_Position = vec4(vertex_pos, 0.0, 1.0);
        screen_pos = vertex_pos;
    }
    \0";

const FS_SRC: &'static [u8] = b"
    #version 100
    precision mediump float;
    uniform sampler2D cols;
    varying vec2 screen_pos;
    void main() {
        gl_FragColor = texture2D(cols, (screen_pos + 1.0) / 2.0);
    }
    \0";


fn draw(buffer: &mut Vec<u8>, w: u32, h: u32, t: f64) {
    let k = (128.0 + f64::cos(t * 3.0) * 128.0) as u8;
    for y in 0..h {
        for x in 0..w {
            buffer[(y * w * 4 + x * 4 + 0) as usize] = (x * 256 / w) as u8;
            buffer[(y * w * 4 + x * 4 + 1) as usize] = (y * 256 / h) as u8;
            buffer[(y * w * 4 + x * 4 + 2) as usize] = k;
            buffer[(y * w * 4 + x * 4 + 3) as usize] = 0xFF;
        }
    }
}

fn run() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_title("Hello, world!").with_dimensions(256, 256);
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    let w : u32 = 256;
    let h : u32 = 256;
    let mut color_buffer : Vec<u8> = Vec::new();
    color_buffer.resize((w * h * 4) as usize, 0);

    unsafe {
        gl_window.make_current().unwrap();

        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.5, 0.5, 0.5, 1.0);

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl::CompileShader(fs);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let mut vb = mem::uninitialized();
        gl::GenBuffers(1, &mut vb);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb);
        gl::BufferData(gl::ARRAY_BUFFER, (VERTEX_DATA.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr, VERTEX_DATA.as_ptr() as *const _, gl::STATIC_DRAW);

        let mut vao = mem::uninitialized();
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

        let pos_attrib = gl::GetAttribLocation(program, b"vertex_pos\0".as_ptr() as *const _);
        gl::VertexAttribPointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0, 0, ptr::null());
        gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
    }


    // Main loop
    let mut running = true;
    let mut last_t = now();
    let mut loops = 0;
    while running {
        // Pools events and reacts to them
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => ()
                },
                _ => ()
            }
        });

        // Draws image to buffer
        draw(&mut color_buffer, w, h, now());

        // Uploads image as a texture and renders it
        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, w as i32, h as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, (&color_buffer[0]) as *const u8 as _);
            gl::ClearColor(0.0, 0.5, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        gl_window.swap_buffers().unwrap();

        // Show fps
        loops += 1;
        let t = now();
        if t > last_t + 1.0 {
            println!("fps: {}", loops);
            loops = 0;
            last_t = now();
        }

        // Wait 25ms
        thread::sleep(Duration::from_millis(10));
    }
}

fn main() {
    run();
}
