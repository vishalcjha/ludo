pub const LUDO_FRAGMENT_SHADER: &str = r#"
precision mediump float;
    varying vec4 v_Color;
    void main() {
        gl_FragColor = v_Color;
    }
"#;
