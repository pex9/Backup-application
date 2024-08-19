use crate::{sys, types::Confirm, types::Point, types::Rectangle};
use std::sync::{Arc, Mutex};
use crate::utils::play_sound;

pub struct Mouse(sys::Mouse);

impl Mouse {
    // Create a new instance of the Mouse
    pub fn new() -> Self {
        Mouse(sys::Mouse::new())
    }

    // Get the current position of the mouse
    pub fn get_position(&self) -> Result<Point, Box<dyn std::error::Error>> {
        self.0.get_position()
    }

    // Wrapper to verify the rectangle is drawn, then we can start the backup
    pub fn rectangle_write(
        &mut self,
        width: i32,
        height: i32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let data = Arc::new(self);
        let mut rect = Rectangle::new(Arc::clone(&data), width, height);
        play_sound("assets/start_command.mp3");
        let res = rect.draw_rectangle();
        if res {
            play_sound("assets/rectangle_drawn.mp3");
        }
        Ok(res)
    }

    pub fn confirm(&mut self,controller: Arc<Mutex<bool>>) -> Result<bool, Box<dyn std::error::Error>> {
        let data = Arc::new(self);
        let mut conf = Confirm::new(Arc::clone(&data));
        play_sound("assets/start_command.mp3");
        let res = conf.confirm(controller);
        Ok(res)
    }
}
