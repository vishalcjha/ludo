pub const VERTEX_SHADER: &str = r##"
    attribute vec4 a_Position;
    uniform mat4 u_xFormMatrix;
    void main() {
        gl_Position = u_xFormMatrix * a_Position;
        // gl_PointSize = 10.0; - commented as point program is converted to triangle. And it does has affect.
    }
"##;
