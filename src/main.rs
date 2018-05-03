#![allow(unreachable_code)]
#![allow(unused_imports)]

#[macro_use]    extern crate glium;
                extern crate crossbeam_channel;


use std::time;

use glium::{glutin, Surface};
use crossbeam_channel::bounded;
use std::thread;



fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new()
        .with_srgb(true);

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

        uniform float time;
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

    let mut fragment_shader_src = String::new();
    fragment_shader_src.push_str(fragment_shader_version);
    fragment_shader_src.push_str(fragment_shader_uniforms);
    fragment_shader_src.push_str(fragment_shader_default);

    println!("{}", fragment_shader_src);


/*
    let fragment_shader_src = r#"
        #version 140

        out vec4 color;


        float plane(vec3 p)
        {
            return p.y;
        }

        float sphere(vec3 p)
        {
            return length(p) - 2.0;
        }

        void main() {
            color = vec4(fract(vPosition), 0.0, 1.0);

            vec2 uv = vPosition;

            // camera origin
            vec3 p = vec3(sin(time) * 4.0, 5.0, -10.0);
            // direction
            vec3 dir = normalize(vec3(uv, 1.0));


            // background color
            color = vec4(fract(uv), 0.0, 1.0);

            for (int i=0; i<1024; i++)
            {
                float d = min(plane(p), sphere(p));

                if (d < 0.000001)
                {
                    color = vec4(fract(p), 1.0);
                    break;
                }
                p += d * dir;
            }

        }
    "#;
    */

    let program = match glium::Program::from_source(&display, vertex_shader_src, &fragment_shader_src, None) {
        Ok(x) => x,
        Err(e) => {
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
                    glutin::WindowEvent::Closed => closed = true,
                    _ => ()
                },
                _ => (),
            }
        });
    }
}
