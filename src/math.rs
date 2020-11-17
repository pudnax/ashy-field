pub(crate) type Vec2 = ultraviolet::Vec2;
pub(crate) type Vec3 = ultraviolet::Vec3;
pub(crate) type Vec4 = ultraviolet::Vec4;
pub(crate) type Mat4 = ultraviolet::Mat4;
pub(crate) type Mat3 = ultraviolet::Mat4;
pub(crate) type Mat2 = ultraviolet::Mat4;
pub(crate) type Bivec3 = ultraviolet::Bivec3;

pub struct PrettyM4<'a>(&'a Mat4);

use std::fmt::{self, Display, Formatter};

impl<'a> Display for PrettyM4<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..4 {
            writeln!(
                f,
                "\t[ {:>3} {:>3} {:>3} {:>3} ]",
                self.0[i][0], self.0[i][1], self.0[i][2], self.0[i][3]
            )?
        }
        Ok(())
    }
}
pub struct PrettyM3<'a>(&'a Mat3);

impl<'a> Display for PrettyM3<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..3 {
            writeln!(
                f,
                "\t[ {:>3} {:>3} {:>3} ]",
                self.0[i][0], self.0[i][1], self.0[i][2]
            )?
        }
        Ok(())
    }
}
pub struct PrettyM2<'a>(&'a Mat2);

impl<'a> Display for PrettyM2<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..2 {
            writeln!(f, "\t[ {:>3} {:>3} ]", self.0[i][0], self.0[i][1])?
        }
        Ok(())
    }
}
