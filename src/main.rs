#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

#[macro_use]
extern crate glium;
extern crate crossbeam_channel;

use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;
use std::thread;
use std::time;

use crossbeam_channel::bounded;
use glium::glutin::surface::WindowSurface;
use glium::{glutin, Surface};

const VERTEX_SHADER_SRC: &str = r#"
    #version 140

    in vec2 i_position;

    out vec2 fragCoord;

    uniform float iTime;
    uniform vec2 res;

    void main() {
        fragCoord = i_position;
        gl_Position = vec4(i_position, 0.0, 1.0);
    }
"#;

mod glsl_graph {
    use std::fmt::Debug;
    use std::fmt::Display;
    use std::rc::Rc;

    pub enum Dimension {
        D1,
        D2,
        D3,
        D4,
    }

    pub trait GlslBLock: Debug {
        fn inputs_dims(&self) -> Vec<Dimension>;
        fn output_dims(&self) -> Dimension;
        fn glsl_code(&self) -> String;
        fn inputs(&self) -> Vec<Rc<dyn GlslBLock>>;
    }

    #[derive(Debug)]
    pub struct BaseSphere {
        input: Rc<dyn GlslBLock>,
    }
    impl BaseSphere {
        pub fn new(input: Rc<dyn GlslBLock>) -> BaseSphere {
            Self { input }
        }
    }
    impl GlslBLock for BaseSphere {
        fn inputs_dims(&self) -> Vec<Dimension> {
            vec![Dimension::D3]
        }
        fn output_dims(&self) -> Dimension {
            Dimension::D1
        }
        fn glsl_code(&self) -> String {
            //float sphere(vec3 p)
            format!("length({}) - 2.0", self.input.glsl_code())
        }
        fn inputs(&self) -> Vec<Rc<dyn GlslBLock>> {
            Vec::new()
        }
    }

    #[derive(Debug)]
    pub struct GlslOut {
        input: Rc<dyn GlslBLock>,
    }
    impl GlslOut {
        pub fn new(input: Rc<dyn GlslBLock>) -> Self {
            Self { input }
        }
    }
    impl GlslBLock for GlslOut {
        fn inputs_dims(&self) -> Vec<Dimension> {
            vec![Dimension::D4]
        }
        fn output_dims(&self) -> Dimension {
            Dimension::D4
        }
        fn glsl_code(&self) -> String {
            format!("fragColor = {};", self.input.glsl_code())
        }
        fn inputs(&self) -> Vec<Rc<dyn GlslBLock>> {
            vec![self.input.clone()]
        }
    }

    #[derive(Debug)]
    pub struct GlslIn {
        name: String,
    }
    impl GlslIn {
        pub fn new(name: &str) -> Self {
            Self { name: name.into() }
        }
    }
    impl GlslBLock for GlslIn {
        fn inputs_dims(&self) -> Vec<Dimension> {
            vec![Dimension::D4]
        }
        fn output_dims(&self) -> Dimension {
            Dimension::D4
        }
        fn glsl_code(&self) -> String {
            self.name.clone()
        }
        fn inputs(&self) -> Vec<Rc<dyn GlslBLock>> {
            Vec::new()
        }
    }
}

struct ShaderToy {
    previous_shader: Option<String>,
    program: glium::Program,
}

impl ShaderToy {
    fn try_load_program_or_fallback(
        display: &glium::Display<WindowSurface>,
        frag_shader_path: &str,
        fallback: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match ShaderToy::load_program(display, frag_shader_path) {
            Ok(x) => Ok(x),
            Err(_) => ShaderToy::load_program(display, fallback),
        }
    }

    fn load_program(
        display: &glium::Display<WindowSurface>,
        frag_shader_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(frag_shader_path)?;
        let mut fragment_shader_src = String::new();
        file.read_to_string(&mut fragment_shader_src)?;

        let source = glium::program::ProgramCreationInput::SourceCode {
            vertex_shader: &VERTEX_SHADER_SRC,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            fragment_shader: &fragment_shader_src,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        };

        let program = glium::Program::new(&display.clone(), source)?;

        Ok(Self {
            previous_shader: Some(fragment_shader_src),
            program,
        })
    }
}

fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Glium tutorial #2")
        .with_inner_size(1024, 768)
        .build(&event_loop);

    #[derive(Copy, Clone)]
    struct Vertex {
        i_position: [f32; 2],
    }

    implement_vertex!(Vertex, i_position);

    let vertex1 = Vertex {
        i_position: [-1.0, -1.0],
    };
    let vertex2 = Vertex {
        i_position: [1.0, -1.0],
    };
    let vertex3 = Vertex {
        i_position: [-1.0, 1.0],
    };
    let vertex4 = Vertex {
        i_position: [1.0, 1.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open("shader.frag").unwrap();
    let mut fragment_shader_src = String::new();
    file.read_to_string(&mut fragment_shader_src).unwrap();

    let current_valid_fragment_shader_source: String = fragment_shader_src.clone();

    let mut durations = Vec::new();

    let clock = time::Instant::now();

    event_loop
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let elapsed = clock.elapsed();
                    let t =
                        elapsed.as_secs() as f32 + elapsed.subsec_micros() as f32 / 1_000_000f32;

                    let res_x = display.get_framebuffer_dimensions().0;
                    let res_y = display.get_framebuffer_dimensions().1;
                    let res = (res_x as f32, res_y as f32);

                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);

                    let uniforms = uniform! {
                        iTime: t,
                        iResolution: res,
                    };

                    let shader_toy = ShaderToy::try_load_program_or_fallback(
                        &display,
                        "shader.frag",
                        "shader_default.frag",
                    )
                    .unwrap();

                    let fps_clock = time::Instant::now();
                    target
                        .draw(
                            &vertex_buffer,
                            &indices,
                            &shader_toy.program,
                            &uniforms,
                            &Default::default(),
                        )
                        .unwrap();
                    target.finish().unwrap();
                    let duration = fps_clock.elapsed().subsec_micros();
                    durations.push(duration);
                }
                _ => (),
            },
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        })
        .unwrap();
}
