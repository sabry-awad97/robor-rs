use std::{fmt, io};
use winapi::{
    shared::windef::POINT,
    um::winuser::{GetCursorPos, GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN},
};

#[derive(Debug)]
pub enum MouseError {
    InvalidInput,
    ConversionError(String),
    IoError(io::Error),
    OutOfBounds,
}

impl fmt::Display for MouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseError::InvalidInput => write!(f, "Invalid input"),
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
        if x < 0 || y < 0 {
            return Err(MouseError::InvalidInput);
        }

        let new_position = MousePosition::new(x, y);
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        unsafe { SetCursorPos(x_u32 as i32, y_u32 as i32) };
        self.position = new_position;
        Ok(())
    }

    pub fn move_relative(&mut self, distance_x: i32, distance_y: i32) -> Result<(), MouseError> {
        let new_x = self.position.x + distance_x;
        let new_y = self.position.y + distance_y;
        self.move_to(new_x, new_y)?;
        Ok(())
    }

    pub fn hover(
        &mut self,
        x: i32,
        y: i32,
        duration: std::time::Duration,
    ) -> Result<(), MouseError> {
        if x < 0 || y < 0 || duration.as_secs() == 0 {
            return Err(MouseError::InvalidInput);
        }
        
        let new_position = MousePosition::new(x, y);
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }

        let current_position = self.get_position();

        let start_x = current_position.0 as f64;
        let start_y = current_position.1 as f64;

        let distance_x = x as f64 - start_x;
        let distance_y = y as f64 - start_y;

        let total_distance = (distance_x.powi(2) + distance_y.powi(2)).sqrt();

        let start_time = std::time::Instant::now();
        while start_time.elapsed() < duration {
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress = elapsed / duration.as_secs_f64();
            let current_distance = total_distance * progress;

            let current_x = ((current_distance / total_distance) * distance_x + start_x) as i32;
            let current_y = ((current_distance / total_distance) * distance_y + start_y) as i32;
            self.move_to(current_x, current_y)?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        self.move_to(x, y)?;
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

    #[test]
    fn test_mouse_move_to_out_of_bounds() {
        let mut mouse = Mouse::new();
        let result = mouse.move_to(-1, 500);
        assert!(result.is_err());
    }

    #[test]
    fn test_move_relative() {
        let mut mouse = Mouse::new();
        mouse.move_to(100, 100).unwrap();
        mouse.move_relative(10, 20).unwrap();
        assert_eq!(mouse.position.x, 110);
        assert_eq!(mouse.position.y, 120);

        mouse.move_relative(-5, -10).unwrap();
        assert_eq!(mouse.position.x, 105);
        assert_eq!(mouse.position.y, 110);
    }

    #[test]
    fn test_move_relative_error() {
        let mut mouse = Mouse::new();
        mouse.move_to(100, 100).unwrap();
        let result = mouse.move_relative(-101, -101);
        assert!(result.is_err());
    }

    #[test]
    fn test_hover_within_bounds() {
        let mut mouse = Mouse::new();
        let result = mouse.hover(50, 50, std::time::Duration::from_secs(1));
        assert!(result.is_ok());
        assert_eq!(mouse.position.x, 50);
        assert_eq!(mouse.position.y, 50);
    }

    #[test]
    fn test_hover_out_of_bounds() {
        let mut mouse = Mouse::new();
        let result = mouse.hover(10000, 10000, std::time::Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_hover_moves_mouse() {
        let mut mouse = Mouse::new();
        let start_position = mouse.get_position();
        let result = mouse.hover(50, 50, std::time::Duration::from_secs(1));
        assert!(result.is_ok());
        let end_position = mouse.get_position();
        assert_ne!(start_position, end_position);
    }
}
