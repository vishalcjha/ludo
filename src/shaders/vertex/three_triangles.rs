pub const THREE_TRIANGLE_SOURCE: &str = r#"
    attribute vec4 a_Position;
    attribute vec4 a_Color;
    varying vec4 v_Color;
    uniform mat4 u_ViewMatrix;
    uniform mat4 u_ProjMatrix;

    void main() {
        gl_Position =  u_ProjMatrix * u_ViewMatrix * a_Position;
        v_Color = a_Color;
    }
"#;
