use crate::buffers::Buffer;
use nalgebra as na;

#[derive(Debug)]
pub struct Camera {
    viewmatrix: na::Matrix4<f32>,
    position: na::Vector3<f32>,
    view_direction: na::Unit<na::Vector3<f32>>,
    down_direction: na::Unit<na::Vector3<f32>>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    projectionmatrix: na::Matrix4<f32>,
}

pub struct CameraBuilder {
    position: na::Vector3<f32>,
    view_direction: na::Unit<na::Vector3<f32>>,
    down_direction: na::Unit<na::Vector3<f32>>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

#[allow(dead_code)]
impl CameraBuilder {
    pub fn build(self) -> Camera {
        if self.far < self.near {
            println!(
                "far plane (at {}) closer than near plane (at {}) — is that right?",
                self.far, self.near
            );
        }
        let mut cam = Camera {
            position: self.position,
            view_direction: self.view_direction,
            down_direction: na::Unit::new_normalize(
                self.down_direction.as_ref()
                    - self
                        .down_direction
                        .as_ref()
                        .dot(self.view_direction.as_ref())
                        * self.view_direction.as_ref(),
            ),
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            viewmatrix: na::Matrix4::identity(),
            projectionmatrix: na::Matrix4::identity(),
        };
        cam.update_projectionmatrix();
        cam.update_viewmatrix();
        cam
    }

    fn position(mut self, pos: na::Vector3<f32>) -> CameraBuilder {
        self.position = pos;
        self
    }

    fn fovy(mut self, fovy: f32) -> CameraBuilder {
        self.fovy = fovy.max(0.01).min(std::f32::consts::PI - 0.01);
        self
    }

    fn aspect(mut self, aspect: f32) -> CameraBuilder {
        self.aspect = aspect;
        self
    }

    fn near(mut self, near: f32) -> CameraBuilder {
        if near <= 0.0 {
            println!("setting near plane to negative value: {} — you sure?", near);
        }
        self.near = near;
        self
    }
    fn far(mut self, far: f32) -> CameraBuilder {
        if far <= 0.0 {
            println!("setting far plane to negative value: {} — you sure?", far);
        }
        self.far = far;
        self
    }
    // TODO(#2): Do nothing if vector is already normalized
    fn view_direction(mut self, direction: na::Vector3<f32>) -> CameraBuilder {
        self.view_direction = na::Unit::new_normalize(direction);
        self
    }
    fn down_direction(mut self, direction: na::Vector3<f32>) -> CameraBuilder {
        self.down_direction = na::Unit::new_normalize(direction);
        self
    }
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            position: na::Vector3::new(3.0, 1.33, 1.33),
            view_direction: na::Unit::new_normalize(na::Vector3::new(-0.81, -0.5, -0.4)),
            down_direction: na::Unit::new_normalize(na::Vector3::new(0.0, 1.0, -1.0)),
            fovy: std::f32::consts::FRAC_PI_3,
            aspect: 800.0 / 600.0,
            near: 0.1,
            far: 100.0,
        }
    }
    fn update_projectionmatrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projectionmatrix = na::Matrix4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }
    pub fn update_buffer(
        &self,
        allocator: &vk_mem::Allocator,
        buffer: &mut Buffer,
    ) -> Result<(), vk_mem::error::Error> {
        let data: [[[f32; 4]; 4]; 2] = [self.viewmatrix.into(), self.projectionmatrix.into()];
        buffer.fill(allocator, &data)?;
        Ok(())
    }

    fn update_viewmatrix(&mut self) {
        let right = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let m = na::Matrix4::new(
            right.x,
            right.y,
            right.z,
            -right.dot(&self.position), //
            self.down_direction.x,
            self.down_direction.y,
            self.down_direction.z,
            -self.down_direction.dot(&self.position), //
            self.view_direction.x,
            self.view_direction.y,
            self.view_direction.z,
            -self.view_direction.dot(&self.position), //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self.viewmatrix = self.projectionmatrix * m;
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += distance * self.view_direction.as_ref();
        self.update_viewmatrix();
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    pub fn turn_right(&mut self, angle: f32) {
        let rotation = na::Rotation3::from_axis_angle(&self.down_direction, angle);
        self.view_direction = rotation * self.view_direction;
        self.update_viewmatrix();
    }

    pub fn turn_left(&mut self, angle: f32) {
        self.turn_right(-angle);
    }

    pub fn turn_up(&mut self, angle: f32) {
        let right = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let rotation = na::Rotation3::from_axis_angle(&right, angle);
        self.view_direction = rotation * self.view_direction;
        self.down_direction = rotation * self.down_direction;
        self.update_viewmatrix();
    }

    pub fn turn_down(&mut self, angle: f32) {
        self.turn_up(-angle);
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projectionmatrix();
    }
}
