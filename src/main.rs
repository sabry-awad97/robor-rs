use robor_rs::Mouse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mouse = Mouse::new();
    mouse.print_mouse_position();
    Ok(())
}
