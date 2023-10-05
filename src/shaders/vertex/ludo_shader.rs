pub const LUDO_VERTEX_SHADER: &str = r#"
    vec4 a_Position;
    void main() {
        gl_Position = a_Position;
    }
"#;
