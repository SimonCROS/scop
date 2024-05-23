use matrix::{Matrix, Vector};

pub type Matrix3 = Matrix<3, 3, f32>;
pub type Matrix4 = Matrix<4, 4, f32>;
pub type Vector2 = Vector<2, f32>;
pub type Vector3 = Vector<3, f32>;
pub type Vector4 = Vector<4, f32>;

pub trait Up {
    fn up() -> Self;
}

pub trait Down {
    fn down() -> Self;
}

pub trait Left {
    fn left() -> Self;
}

pub trait Right {
    fn right() -> Self;
}

pub trait Forward {
    fn forward() -> Self;
}

pub trait Backward {
    fn backward() -> Self;
}

macro_rules! define_directions {
    ($Vector:ty, $up:expr, $down:expr, $left:expr, $right:expr) => {
        impl Up for $Vector {
            fn up() -> Self {
                $up.into()
            }
        }

        impl Down for $Vector {
            fn down() -> Self {
                $down.into()
            }
        }

        impl Left for $Vector {
            fn left() -> Self {
                $left.into()
            }
        }

        impl Right for $Vector {
            fn right() -> Self {
                $right.into()
            }
        }
    };
}

define_directions!(Vector2, [0., 1.], [0., -1.], [-1., 0.], [1., 0.]);
define_directions!(
    Vector3,
    [0., 1., 0.],
    [0., -1., 0.],
    [-1., 0., 0.],
    [1., 0., 0.]
);

impl Forward for Vector3 {
    fn forward() -> Self {
        [0., 0., 1.].into()
    }
}

impl Backward for Vector3 {
    fn backward() -> Self {
        [0., 0., -1.].into()
    }
}
