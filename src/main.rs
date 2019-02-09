#![allow(unreachable_code)]
#![allow(unused_imports)]

#[macro_use]    extern crate glium;
                extern crate crossbeam_channel;


use std::time;
use std::thread;
use std::rc::Rc;
use std::fmt::Debug;
use std::fmt::Display;



use glium::{glutin, Surface};
use crossbeam_channel::bounded;


mod glsl_graph {
    use std::rc::Rc;
    use std::fmt::Debug;
    use std::fmt::Display;

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
        fn inputs(&self) -> Vec<Rc<GlslBLock>>;
    }

    #[derive(Debug)]
    pub struct BaseSphere {
        input: Rc<GlslBLock>,
    }
    impl BaseSphere {
        pub fn new(input: Rc<GlslBLock>) -> BaseSphere {
            Self {
                input
            }
        }
    }
    impl GlslBLock for BaseSphere {
        fn inputs_dims(&self) -> Vec<Dimension> { vec![Dimension::D3] }
        fn output_dims(&self) -> Dimension { Dimension::D1 }
        fn glsl_code(&self) -> String {
            //float sphere(vec3 p)
            format!("length({}) - 2.0", self.input.glsl_code())
        }
        fn inputs(&self) -> Vec<Rc<GlslBLock>> {
            Vec::new()
        }
    }

    #[derive(Debug)]
    pub struct GlslOut {
        input: Rc<GlslBLock>,
    }
    impl GlslOut {
        pub fn new(input: Rc<GlslBLock>) -> Self {
            Self {
                input,
            }
        }
    }
    impl GlslBLock for GlslOut {
        fn inputs_dims(&self) -> Vec<Dimension> { vec![Dimension::D4] }
        fn output_dims(&self) -> Dimension { Dimension::D4 }
        fn glsl_code(&self) -> String {
            format!("fragColor = {};", self.input.glsl_code())
        }
        fn inputs(&self) -> Vec<Rc<GlslBLock>> {
            vec![self.input.clone()]
        }
    }

    #[derive(Debug)]
    pub struct GlslIn {
        name: String,
    }
    impl GlslIn {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.into(),
            }
        }
    }
    impl GlslBLock for GlslIn {
        fn inputs_dims(&self) -> Vec<Dimension> { vec![Dimension::D4] }
        fn output_dims(&self) -> Dimension { Dimension::D4 }
        fn glsl_code(&self) -> String {
            self.name.clone()
        }
        fn inputs(&self) -> Vec<Rc<GlslBLock>> {
            Vec::new()
        }
    }
}



fn main() {
    {
        use glsl_graph::*;

        let frag_coord = Rc::new(GlslIn::new("fragCoord"));
        let sphere = Rc::new(BaseSphere::new(frag_coord.clone()));
        let frag_color = Rc::new(GlslOut::new(sphere));

        let mut frag_shader = String::new();
        frag_shader += "#version 140\n";
        frag_shader += r#"
uniform vec2      iResolution;           // viewport resolution (in pixels)
uniform float     iTime;                 // shader playback time (in seconds)
uniform float     iTimeDelta;            // render time (in seconds)
uniform int       iFrame;                // shader playback frame
uniform float     iChannelTime[4];       // channel playback time (in seconds)
uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
//uniform sampler2D iChannel0..3;          // input channel. XX = 2D/Cube
uniform vec4      iDate;                 // (year, month, day, time in seconds)
uniform float     iSampleRate;           // sound sample rate (i.e., 44100)

in vec2 fragCoord;
out vec4 fragColor;

void main()
{
"#;
        let frag_code = frag_color.glsl_code();

        frag_shader += &frag_code;
        frag_shader += r#"
}
"#;
    }


    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();

    let display = glium::Display::new(window, context, &events_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        i_position: [f32; 2],
    }

    implement_vertex!(Vertex, i_position);

    let vertex1 = Vertex { i_position: [-1.0, -1.0] };
    let vertex2 = Vertex { i_position: [ 1.0, -1.0] };
    let vertex3 = Vertex { i_position: [-1.0,  1.0] };
    let vertex4 = Vertex { i_position: [ 1.0,  1.0] };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let vertex_shader_src = r#"
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

    let fragment_shader_version = "#version 140\n";
    let fragment_shader_uniforms = r#"
uniform vec2      iResolution;           // viewport resolution (in pixels)
uniform float     iTime;                 // shader playback time (in seconds)
uniform float     iTimeDelta;            // render time (in seconds)
uniform int       iFrame;                // shader playback frame
uniform float     iChannelTime[4];       // channel playback time (in seconds)
uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
//uniform sampler2D iChannel0..3;          // input channel. XX = 2D/Cube
uniform vec4      iDate;                 // (year, month, day, time in seconds)
uniform float     iSampleRate;           // sound sample rate (i.e., 44100)

in vec2 fragCoord;
out vec4 fragColor;
"#;
    

    let fragment_shader_default = r#"
void main()
{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord;  // [ -1, 1]
    //vec2 uv = (fragCoord + 1.0) / 2.0;   // [0, 1]
 
    // Time varying pixel color
    vec3 col = vec3(fract(uv), 0.0);
    //vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    //col = vec3(1.0, 1.0, 0.0);

    // Output to screen
    fragColor = vec4(col,1.0);
}
"#;

    //let mut fragment_shader_src = String::new();
    //fragment_shader_src.push_str(fragment_shader_version);
    //fragment_shader_src.push_str(fragment_shader_uniforms);
    //fragment_shader_src.push_str(fragment_shader_default);


    let fragment_shader_src = r#"
        #version 140

        uniform vec2      iResolution;           // viewport resolution (in pixels)
        uniform float     iTime;                 // shader playback time (in seconds)
        uniform float     iTimeDelta;            // render time (in seconds)
        uniform int       iFrame;                // shader playback frame
        uniform float     iChannelTime[4];       // channel playback time (in seconds)
        uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
        uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
        //uniform sampler2D iChannel0..3;          // input channel. XX = 2D/Cube
        uniform vec4      iDate;                 // (year, month, day, time in seconds)
        uniform float     iSampleRate;           // sound sample rate (i.e., 44100)
        
        in vec2 fragCoord;
        out vec4 fragColor;


        float plane(vec3 p)
        {
            return p.y;
        }

        float sphere(vec3 p)
        {
            return length(p) - 2.0;
        }

        void main() {
            vec2 uv = fragCoord;

            // camera origin
            vec3 p = vec3(sin(iTime) * 4.0, 5.0, -10.0);
            // direction
            vec3 dir = normalize(vec3(uv, 1.0));


            // background color
            fragColor = vec4(fract(uv), 0.0, 1.0);

            for (int i=0; i<1024; i++)
            {
                float d = min(plane(p), sphere(p));

                if (d < 0.000001)
                {
                    fragColor = vec4(fract(p), 1.0);
                    break;
                }
                p += d * dir;
            }

        }
    "#;
    


    let source = glium::program::ProgramCreationInput::SourceCode {
        vertex_shader: &vertex_shader_src,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: &fragment_shader_src,
        transform_feedback_varyings: None,
        outputs_srgb: true,
        uses_point_size: false,
    };

    let program = match glium::Program::new(&display, source) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", fragment_shader_src);
            println!("{}", e);
            panic!();
        },
    };
    

    let mut closed = false;
    let clock = time::Instant::now();
    while !closed {
        let elapsed = clock.elapsed();
        let t = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1_000_000_000f32;

        let res_x = display.get_framebuffer_dimensions().0;
        let res_y = display.get_framebuffer_dimensions().1;
        let res = (res_x as f32, res_y as f32);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let uniforms = uniform! {
            iTime: t,
            iResolution: res,
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => closed = true,
                    _ => ()
                },
                _ => (),
            }
        });
    }
}
