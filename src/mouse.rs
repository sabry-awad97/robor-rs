use std::{
    fmt::{self, Display},
    io,
};
use winapi::{
    shared::windef::POINT,
    um::winuser::{
        mouse_event, GetAsyncKeyState, GetCursorPos, GetSystemMetrics, SendInput, SetCursorPos,
        INPUT, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
        MOUSEEVENTF_WHEEL, SM_CXSCREEN, SM_CYSCREEN, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
    },
};

use crate::event_emitter::EventEmitter;

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

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum ButtonAction {
    Press,
    Release,
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

    pub fn offset(&self, distance_x: i32, distance_y: i32) -> Self {
        let x = self.x + distance_x;
        let y = self.y + distance_y;

        Self::new(x, y)
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

pub enum EventType {
    Click,
}

impl Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Click => write!(f, "Click"),
        }
    }
}

pub struct Mouse {
    position: MousePosition,
    event_emitter: EventEmitter,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            position: MousePosition::default(),
            event_emitter: EventEmitter::new(),
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

    pub fn move_in_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: i32,
        duration: std::time::Duration,
    ) -> Result<(), MouseError> {
        if radius <= 0 || duration.as_secs() == 0 {
            return Err(MouseError::InvalidInput);
        }

        let start_time = std::time::Instant::now();
        while start_time.elapsed() < duration {
            let elapsed = start_time.elapsed().as_secs_f64();
            let angle = elapsed * 2.0 * std::f64::consts::PI / duration.as_secs_f64();
            let x = center_x + (radius as f64 * angle.cos()) as i32;
            let y = center_y + (radius as f64 * angle.sin()) as i32;
            self.move_to(x, y)?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }

    pub fn on<F>(&mut self, event_type: EventType, listener: F)
    where
        F: Fn() + 'static + Send + Sync,
    {
        let event_name = &event_type.to_string();
        self.event_emitter.on(event_name, listener);
    }

    pub fn click(&mut self) -> Result<(), MouseError> {
        let new_position = &self.position;
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, x_u32, y_u32, 0, 0) };
        unsafe { mouse_event(MOUSEEVENTF_LEFTUP, x_u32, y_u32, 0, 0) };
        self.event_emitter.emit(&EventType::Click.to_string());
        Ok(())
    }

    pub fn double_click(&mut self) -> Result<(), MouseError> {
        self.click()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.click()
    }

    pub fn multi_click(&mut self, count: usize) -> Result<(), MouseError> {
        let new_position = &self.position;
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        for _ in 0..count {
            unsafe {
                mouse_event(MOUSEEVENTF_LEFTDOWN, x_u32, y_u32, 0, 0);
                mouse_event(MOUSEEVENTF_LEFTUP, x_u32, y_u32, 0, 0)
            };
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Ok(())
    }

    pub fn right_click(&mut self) -> Result<(), MouseError> {
        let new_position = &self.position;
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        unsafe { mouse_event(MOUSEEVENTF_RIGHTDOWN, x_u32, y_u32, 0, 0) };
        unsafe { mouse_event(MOUSEEVENTF_RIGHTUP, x_u32, y_u32, 0, 0) };
        Ok(())
    }

    pub fn scroll(&mut self, amount: i32) -> Result<(), MouseError> {
        let new_position = &self.position;
        if new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = new_position.to_u32()?;
        unsafe { mouse_event(MOUSEEVENTF_WHEEL, x_u32, y_u32, amount as u32, 0) };
        Ok(())
    }

    pub fn scroll_horizontal(&mut self, distance: i32) -> Result<(), MouseError> {
        let params = [0, 0, 0, distance as u32];
        unsafe {
            mouse_event(MOUSEEVENTF_HWHEEL, 0, 0, params[3] as u32, 0);
        }
        Ok(())
    }

    pub fn scroll_with_delay(
        &mut self,
        amount: i32,
        delay: std::time::Duration,
    ) -> Result<(), MouseError> {
        if self.position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }
        let (x_u32, y_u32) = self.position.to_u32()?;
        let step = amount.signum();
        let mut remaining = amount.abs();
        while remaining != 0 {
            let scroll_amount = std::cmp::min(remaining, step.abs());
            let scroll_direction = if amount < 0 { -1 } else { 1 };
            unsafe {
                mouse_event(
                    MOUSEEVENTF_WHEEL,
                    x_u32,
                    y_u32,
                    (scroll_amount * scroll_direction) as u32,
                    0,
                )
            };
            remaining -= scroll_amount;
            std::thread::sleep(delay);
        }
        Ok(())
    }

    pub fn drag_and_drop(&mut self, distance_x: i32, distance_y: i32) -> Result<(), MouseError> {
        let current_position = &self.position;
        let new_position = current_position.offset(distance_x, distance_y);

        if current_position.is_out_of_bounds() || new_position.is_out_of_bounds() {
            return Err(MouseError::OutOfBounds);
        }

        let (current_x, current_y) = current_position.to_u32()?;
        let (new_x, new_y) = new_position.to_u32()?;

        unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, current_x, current_y, 0, 0) };
        unsafe { mouse_event(MOUSEEVENTF_LEFTUP, new_x, new_y, 0, 0) };
        self.position = new_position;

        Ok(())
    }

    pub fn simulate_mouse_button(&self, button: MouseButton, action: ButtonAction) {
        let (down_flag, up_flag) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };
        let flags = match action {
            ButtonAction::Press => down_flag,
            ButtonAction::Release => up_flag,
        };
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: unsafe { std::mem::zeroed() },
        };
        unsafe {
            input.u.mi_mut().dwFlags = flags;
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn is_left_button_pressed(&self) -> bool {
        let state = unsafe { GetAsyncKeyState(VK_LBUTTON) } as u32;
        state & 0x8001 != 0
    }

    pub fn is_right_button_pressed(&self) -> bool {
        let state = unsafe { GetAsyncKeyState(VK_RBUTTON) } as u32;
        state & 0x8001 != 0
    }

    pub fn is_middle_button_pressed(&self) -> bool {
        let state = unsafe { GetAsyncKeyState(VK_MBUTTON) } as u32;
        state & 0x8001 != 0
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
    fn test_mouse_position_offset() {
        let position = MousePosition::new(100, 100);
        let offset_position = position.offset(50, -50);

        assert_eq!(offset_position.x, 150);
        assert_eq!(offset_position.y, 50);
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

    #[test]
    fn test_move_in_circle() {
        let mut mouse = Mouse::new();
        let center_x = 100;
        let center_y = 100;
        let radius = 50;
        let duration = std::time::Duration::from_secs(1);
        let result = mouse.move_in_circle(center_x, center_y, radius, duration);
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_in_circle_invalid_radius() {
        let mut mouse = Mouse::new();
        let result = mouse.move_in_circle(0, 0, 0, std::time::Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_move_in_circle_invalid_duration() {
        let mut mouse = Mouse::new();
        let result = mouse.move_in_circle(0, 0, 50, std::time::Duration::from_secs(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_click_within_bounds() {
        let mut mouse = Mouse::new();
        assert!(mouse.click().is_ok());
    }

    #[test]
    fn test_double_click() {
        let mut mouse = Mouse::new();
        assert!(mouse.double_click().is_ok());
    }

    #[test]
    fn test_multi_click_within_bounds() {
        let mut mouse = Mouse::new();
        mouse.move_to(100, 100).unwrap();
        assert!(mouse.multi_click(3).is_ok());
    }

    #[test]
    fn test_right_click() {
        let mut mouse = Mouse::new();
        mouse.move_to(100, 100).unwrap();
        assert!(mouse.right_click().is_ok());
    }

    #[test]
    fn test_scroll() {
        let mut mouse = Mouse::new();
        mouse.move_to(800, 800).unwrap();
        assert!(mouse.scroll(-120).is_ok());
    }

    #[test]
    fn test_scroll_horizontal_positive_distance() {
        let mut mouse = Mouse::new();
        assert!(mouse.scroll_horizontal(10).is_ok());
    }

    #[test]
    fn test_scroll_horizontal_negative_distance() {
        let mut mouse = Mouse::new();
        assert!(mouse.scroll_horizontal(-5).is_ok());
    }

    #[test]
    fn test_scroll_with_delay_within_bounds() {
        let mut mouse = Mouse::new();
        let result = mouse.scroll_with_delay(120, std::time::Duration::from_millis(10));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scroll_with_delay_zero_amount() {
        let mut mouse = Mouse::new();
        let result = mouse.scroll_with_delay(0, std::time::Duration::from_millis(10));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scroll_with_delay_positive_amount() {
        let mut mouse = Mouse::new();
        let result = mouse.scroll_with_delay(120, std::time::Duration::from_millis(10));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scroll_with_delay_negative_amount() {
        let mut mouse = Mouse::new();
        let result = mouse.scroll_with_delay(-120, std::time::Duration::from_millis(10));
        assert!(result.is_ok());
    }

    #[test]
    fn test_drag_and_drop_within_bounds() {
        let mut mouse = Mouse::new();
        mouse.move_to(100, 100).unwrap();
        mouse.drag_and_drop(50, 50).unwrap();
        let (x, y) = mouse.get_position();
        assert_eq!(x, 150);
        assert_eq!(y, 150);
    }

    #[test]
    fn test_simulate_left_button_press() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Left, ButtonAction::Press);
        assert!(
            mouse.is_left_button_pressed(),
            "Left button should be pressed"
        );
    }

    #[test]
    fn test_simulate_left_button_release() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Left, ButtonAction::Release);
        assert!(
            !mouse.is_left_button_pressed(),
            "Left button should be released"
        );
    }

    #[test]
    fn test_simulate_right_button_press() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Right, ButtonAction::Press);
        assert!(
            mouse.is_right_button_pressed(),
            "Right button should be pressed"
        );
    }

    #[test]
    fn test_simulate_right_button_release() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Right, ButtonAction::Release);
        assert!(
            !mouse.is_right_button_pressed(),
            "Right button should be released"
        );
    }

    #[test]
    fn test_simulate_middle_button_press() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Middle, ButtonAction::Press);
        assert!(
            mouse.is_middle_button_pressed(),
            "Middle button should be pressed"
        );
    }

    #[test]
    fn test_simulate_middle_button_release() {
        let mouse = Mouse::new();
        mouse.simulate_mouse_button(MouseButton::Middle, ButtonAction::Release);
        assert!(
            !mouse.is_middle_button_pressed(),
            "Middle button should be released"
        );
    }
}
