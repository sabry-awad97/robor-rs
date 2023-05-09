use std::{fmt, io};
use winapi::{
    shared::windef::POINT,
    um::winuser::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SetCursorPos},
};

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

    pub fn is_out_of_bounds(&self) -> bool {
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        self.x < 0 || self.y < 0 || self.x > screen_width || self.y > screen_height
    }

    pub fn to_u32(&self) -> Result<(u32, u32), MouseError> {
        let x_u32 = self
            .x
            .try_into()
            .map_err(|_| MouseError::ConversionError("Failed to convert x to u32".to_string()))?;
        let y_u32 = self
            .y
            .try_into()
            .map_err(|_| MouseError::ConversionError("Failed to convert y to u32".to_string()))?;
        Ok((x_u32, y_u32))
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

    pub fn get_position(&self) -> (i32, i32) {
        (self.position.x, self.position.y)
    }

    pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), MouseError> {
        let new_position = MousePosition::new(x, y);
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        unsafe { SetCursorPos(x_u32 as i32, y_u32 as i32) };
        self.position = new_position;
        Ok(())
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
    fn test_is_out_of_bounds() {
        let mouse_pos = MousePosition::new(-10, 20);
        assert!(mouse_pos.is_out_of_bounds());

        let mouse_pos = MousePosition::new(10, -20);
        assert!(mouse_pos.is_out_of_bounds());

        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        let mouse_pos = MousePosition::new(screen_width + 10, screen_height + 20);
        assert!(mouse_pos.is_out_of_bounds());

        let mouse_pos = MousePosition::new(screen_width - 10, screen_height - 20);
        assert!(!mouse_pos.is_out_of_bounds());
    }

    #[test]
    fn test_mouse_position_to_u32() {
        let mouse_pos = MousePosition::new(10, 20);
        let result = mouse_pos.to_u32();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (10u32, 20u32));

        let mouse_pos = MousePosition::new(-10, 20);
        assert!(mouse_pos.to_u32().is_err());

        let mouse_pos = MousePosition::new(10, -20);
        assert!(mouse_pos.to_u32().is_err());
    }

    #[test]
    fn test_mouse_position_to_u32_conversion_error() {
        let position = MousePosition::new(-1, 500);
        let result = position.to_u32();
        assert!(result.is_err());
    }

    #[test]
    fn test_mouse_default_position() {
        let position = MousePosition::default();
        assert!(!position.is_out_of_bounds());
    }

    #[test]
    fn test_mouse_new() {
        let mouse = Mouse::new();
        assert!(mouse.position.x >= 0);
        assert!(mouse.position.y >= 0);
    }

    #[test]
    fn test_mouse_get_position() {
        let mouse = Mouse::new();
        let (x, y) = mouse.get_position();
        assert!(x >= 0 && y >= 0);
    }

    #[test]
    fn test_mouse_move_to() {
        let mut mouse = Mouse::new();
        let result = mouse.move_to(100, 200);
        assert!(result.is_ok());
        assert_eq!(mouse.get_position(), (100, 200));
    }
}
