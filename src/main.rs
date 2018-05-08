#[macro_use]
extern crate glium;
extern crate image;
use glium::{glutin, Surface};

mod teapot;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &teapot::INDICES).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;
        out vec3 v_position;

        uniform mat4 matrix1;
        uniform mat4 matrix2;
        uniform mat4 matrix3;
        uniform mat4 normalizer;
        uniform mat4 perspective;
        uniform mat4 dislocator;

        void main() {
            mat4 matrix = matrix1 * matrix2 * matrix3 * normalizer;
            v_normal = transpose(inverse(mat3(matrix))) * normal;
            gl_Position = (dislocator + perspective * matrix) * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        in vec3 v_position;
        out vec4 color;

        uniform vec3 u_light;

        const vec3 ambient_color = vec3(0.2, 0.0, 0.0);
        const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
        const vec3 specular_color = vec3(1.0, 1.0, 1.0);

        void main() {
            float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light) + camera_dir);
            float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

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

        let normalizer = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
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
            normalizer: normalizer,
            perspective: perspective,
            dislocator: dislocator,
            u_light: light,
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.draw((&positions, &normals), &indices, &program, &uniforms, &params).unwrap();
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
