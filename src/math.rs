pub(crate) type Vec2 = ultraviolet::Vec2;
pub(crate) type Vec4 = ultraviolet::Vec4;
pub(crate) type Mat4 = ultraviolet::Mat4;
pub(crate) type Mat3 = ultraviolet::Mat4;
pub(crate) type Mat2 = ultraviolet::Mat4;

pub struct PrettyM4(Mat4);

use std::fmt::{self, Display, Formatter};

impl Display for PrettyM4 {
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
pub struct PrettyM3(Mat3);

impl Display for PrettyM3 {
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
pub struct PrettyM2(Mat2);

impl Display for PrettyM2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..2 {
            writeln!(f, "\t[ {:>3} {:>3} ]", self.0[i][0], self.0[i][1])?
        }
        Ok(())
    }
}
