use std::time::Duration;

use robor_rs::Mouse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mouse = Mouse::new();
    // Wait for 5 seconds.
    std::thread::sleep(Duration::from_secs(5));

    mouse.move_to(300, 400)?;
    // Click to make the window active.
    mouse.click()?;

    let mut distance = 300;
    let change = 20;
    let duration = Duration::from_millis(200);
    while distance > 0 {
        // Move right.
        mouse.drag_with_duration(distance, 0, duration)?;
        distance -= change;
        // Move down.
        mouse.drag_with_duration(0, distance, duration)?;
        // Move left.
        mouse.drag_with_duration(-distance, 0, duration)?;
        distance -= change;
        // Move up.
        mouse.drag_with_duration(0, -distance, duration)?;
    }
    Ok(())
}
