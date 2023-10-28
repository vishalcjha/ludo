use nalgebra::{Matrix4, Vector3};

const GRAVITY: Vector3<f32> = Vector3::new(0., 0.0001, 0.);
#[derive(Debug)]
pub(super) struct DiceState {
    velocity: Vector3<f32>,
    rotation: f32,
    position: Vector3<f32>,
    gravity: Vector3<f32>,
    original_position: Vector3<f32>,
}

impl DiceState {
    pub(super) fn new(original_position: &Vector3<f32>) -> Self {
        let mut dice = DiceState {
            position: Vector3::new(0.0, 0., 0.),    // Initial position
            velocity: Vector3::new(0.0, 0.0, 0.02), // Initial velocity (starts rolling forward)
            rotation: 0.0,                          // Initial rotation
            gravity: Vector3::new(0., -0.01, 0.),
            original_position: Vector3::new(0.0, 0., 0.),
        };

        // Apply some randomness to the initial velocity and rotation
        // dice.velocity.x += (random() * 0.2 - 0.1) as f32; // Random X velocity
        // dice.velocity.y += (random() * 0.02) as f32; // Random Y velocity
        // dice.rotation -= (random() * 0.1) as f32; // Random initial rotation

        dice
    }

    fn update_state(&mut self) {
        self.position += self.velocity;
        self.velocity -= self.gravity;
        self.rotation += self.velocity.norm();
    }

    pub fn get_updated_position(&mut self) -> Matrix4<f32> {
        self.update_state();
        let y_diff = self.position.y - self.original_position.y;
        if y_diff > 1.0 {
            self.gravity = Vector3::new(0., 0.01, 0.);
        } else if y_diff < -1.0 {
            self.gravity = Vector3::new(0., -0.01, 0.);
        }

        let z_diff = self.position.z - self.original_position.z;
        if f32::abs(z_diff) > 10. {
            self.reset();
        }

        let translation = Matrix4::new_translation(&self.position);
        let rotation = Matrix4::from_euler_angles(self.rotation.clone(), 0., 0.);

        translation * rotation
        // Matrix4::from_row_slice(&[
        //     0.0, 6., -13., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
        // ])
        // Matrix4::identity()
    }

    pub fn has_stopped(&self) -> bool {
        self.velocity.norm() < 0.01
    }

    pub fn reset(&mut self) {
        let original_position = self.original_position;
        *self = DiceState::new(&original_position);
    }

    pub fn reset_if_stopped(&mut self) {
        if self.has_stopped() {
            self.reset();
        }
    }
}
