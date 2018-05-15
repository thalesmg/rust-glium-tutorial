#[macro_use]
extern crate glium;
extern crate image;
use glium::{glutin, Surface};

use std::io::Cursor;

mod teapot;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coords);

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let shape = glium::vertex::VertexBuffer::new(&display, &[
        Vertex{position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0]},
        Vertex{position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0]},
        Vertex{position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0]},
        Vertex{position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0]},
    ]).unwrap();

    // texture
    let image = image::load(Cursor::new(&include_bytes!("./tuto-14-diffuse.jpg")[..]), image::JPEG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let image = image::load(Cursor::new(&include_bytes!("./tuto-14-normal.png")[..]), image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let normal_map = glium::texture::Texture2d::new(&display, image).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;
        in vec2 tex_coords;

        out vec3 v_normal;
        out vec3 v_position;
        out vec2 v_tex_coords;

        uniform mat4 matrix1;
        uniform mat4 matrix2;
        uniform mat4 matrix3;
        uniform mat4 wall_view_matrix;
        uniform mat4 normalizer;
        uniform mat4 perspective;
        uniform mat4 dislocator;

        void main() {
            v_tex_coords = tex_coords;
            mat4 matrix = matrix1 * matrix2 * matrix3 * normalizer;
            // mat4 matrix = wall_view_matrix;
            v_normal = transpose(inverse(mat3(matrix))) * normal;
            gl_Position = (dislocator + perspective * matrix) * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        in vec3 v_position;
        in vec2 v_tex_coords;
        out vec4 color;

        uniform vec3 u_light;
        uniform sampler2D diffuse_tex;
        uniform sampler2D normal_tex;

        mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
            vec3 dp1 = dFdx(pos);
            vec3 dp2 = dFdy(pos);
            vec2 duv1 = dFdx(uv);
            vec2 duv2 = dFdy(uv);

            vec3 dp2perp = cross(dp2, normal);
            vec3 dp1perp = cross(normal, dp1);
            vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
            vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

            float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
            return mat3(T * invmax, B * invmax, normal);
        }

        void main() {
            vec3 diffuse_color = texture(diffuse_tex, v_tex_coords).rgb;
            vec3 ambient_color = diffuse_color * 0.1;
            const vec3 specular_color = vec3(1.0, 1.0, 1.0);

            vec3 normal_map = texture(normal_tex, v_tex_coords).rgb;
            mat3 tbn = cotangent_frame(v_normal, v_position, v_tex_coords);
            vec3 real_normal = normalize(tbn * -(normal_map * 2.0 - 1.0));

            float diffuse = max(dot(normalize(real_normal), normalize(u_light)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light) + camera_dir);
            float specular = pow(max(dot(half_direction, normalize(real_normal)), 0.0), 16.0);

            //vec3 dark_color = vec3(0.6, 0.0, 0.0);
            //vec3 regular_color = vec3(1.0, 0.0, 0.0);
            //color = vec4(mix(dark_color, regular_color, brightness), 1.0);

            color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let light = [1.4, 0.4, -0.7f32];

    let mut closed = false;

    let mut tetas: Vec<f32> = vec![0.0, 0.0, 0.0];
    let step = 0.02;

    while !closed {
        for teta in tetas.iter_mut() {
            *teta += step;
            if teta > &mut (2.0 * std::f32::consts::PI) {
                *teta = 0.0;
            }
        }
        let teta1 = tetas[0];
        let teta2 = tetas[1];
        let teta3 = tetas[2];

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let scale = 1.0;
        let normalizer = [
            [scale, 0.0, 0.0, 0.0],
            [0.0, scale, 0.0, 0.0],
            [0.0, 0.0, scale, 0.0],
            [0.0, 0.0, 2.0, 1.0f32],
        ];
        let dislocator = [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0f32],
        ];
        let matrix1 = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, teta1.cos(), teta1.sin(), 0.0],
            [0.0, -teta1.sin(), teta1.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];
        let matrix2 = [
            [teta2.cos(), 0.0, teta2.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-teta2.sin(), 0.0, teta2.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];
        let matrix3 = [
            [teta3.cos(), teta3.sin(), 0.0, 0.0],
            [-teta3.sin(), teta3.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        let wall_view_matrix = view_matrix(
            &[0.5, 0.2, 0.3], &[-0.5, -0.2, 3.0], &[0.0, 0.1, 0.0]
        );

        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f * aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]
            ]
        };

        let uniforms = uniform! {
            matrix1: matrix1,
            matrix2: matrix2,
            matrix3: matrix3,
            wall_view_matrix: wall_view_matrix,
            normalizer: normalizer,
            perspective: perspective,
            dislocator: dislocator,
            u_light: light,
            diffuse_tex: &diffuse_texture,
            normal_tex: &normal_map,
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.draw(&shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program, &uniforms, &params).unwrap();
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

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}
