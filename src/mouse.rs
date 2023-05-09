use std::{fmt, io};
use winapi::{shared::windef::POINT, um::winuser::GetCursorPos};

#[derive(Debug)]
pub enum MouseError {
    ConversionError(String),
    IoError(io::Error),
    OutOfBounds,
}

impl fmt::Display for MouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            MouseError::IoError(err) => write!(f, "IO error: {}", err),
            MouseError::OutOfBounds => write!(f, "Mouse position out of bounds"),
        }
    }
}

impl From<io::Error> for MouseError {
    fn from(err: io::Error) -> Self {
        MouseError::IoError(err)
    }
}

pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

impl MousePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Default for MousePosition {
    fn default() -> Self {
        unsafe {
            let mut point: POINT = std::mem::zeroed();
            GetCursorPos(&mut point);
            Self::new(point.x, point.y)
        }
    }
}

pub struct Mouse {
    position: MousePosition,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            position: MousePosition::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_position_new() {
        let position = MousePosition::new(10, 20);
        assert_eq!(position.x, 10);
        assert_eq!(position.y, 20);
    }

    #[test]
    fn test_mouse_position_default() {
        let position = MousePosition::default();
        assert!(position.x >= 0);
        assert!(position.y >= 0);
    }

    #[test]
    fn test_mouse_new() {
        let mouse = Mouse::new();
        assert!(mouse.position.x >= 0);
        assert!(mouse.position.y >= 0);
    }
}
