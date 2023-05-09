pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

impl MousePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mouse_pos = MousePosition::new(10, 20);
        assert_eq!(mouse_pos.x, 10);
        assert_eq!(mouse_pos.y, 20);
    }
}
