use crate::types::keys::Keys;
use crate::{sys, types::Confirm, types::Point, types::Rectangle};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

static CLICK_COUNT: AtomicU32 = AtomicU32::new(0);

const TOL: i32 = 50;
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Positive,
    Negative,
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Direction::Positive => match other {
                Direction::Positive => true,
                Direction::Negative => false,
            },
            Direction::Negative => match other {
                Direction::Negative => true,
                Direction::Positive => false,
            },
        }
    }
}

impl Eq for Direction {}

pub struct Mouse(sys::Mouse);

impl Mouse {
    // Create a new instance of the Mouse
    pub fn new() -> Self {
        Mouse(sys::Mouse::new())
    }

    // Move the mouse cursor to the specified position
    pub fn move_to(&self, x: i32, y: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.0.move_to(x, y)
    }

    // Press the button of the mouse until it is released
    pub fn press<'a>(&self, button: &'a Keys) -> Result<(), Box<dyn std::error::Error + 'a>> {
        self.track_click();
        self.0.press(button)
    }

    // Relace the button of the mouse
    pub fn release<'a>(&self, button: &'a Keys) -> Result<(), Box<dyn std::error::Error + 'a>> {
        self.0.release(button)
    }

    // Get the current position of the mouse
    pub fn get_position(&self) -> Result<Point, Box<dyn std::error::Error>> {
        self.0.get_position()
    }

    // This will scroll the mouse, scroll down is negative, scroll up is positive
    pub fn wheel(&self, delta: i32) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Clicking for the {:?} time",
            CLICK_COUNT.load(Ordering::Relaxed)
        );
        self.0.wheel(delta)
    }

    // Press and release the button in one function
    pub fn click<'a>(&self, button: &'a Keys) -> Result<(), Box<dyn std::error::Error + 'a>> {
        self.0.press(button).unwrap_or(());
        self.0.release(button)
    }

    // Wrapper to verify the rectangle is drawn, then we can start the backup
    pub fn rectangle_write(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let data = Arc::new(self);
        let mut rect = Rectangle::new(Arc::clone(&data), width, height);
        let res = rect.draw_rectangle();
        match res {
            true => println!("Rectangle drawn"), // In final version, we will start the backup here
            false => println!("We weren't drawing the rectangle"), // In final version, we will do nothing here
        }
        Ok(res)
    }

    pub fn confirm(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(self);
        let mut conf = Confirm::new(Arc::clone(&data));
        match conf.confirm() {
            true => println!("Confirmed"), // In final version, we will confirm the backup here gui to insert and activate backup
            false => println!("We weren't confirming"), // In final version, we will cancel the backup here
        }
        Ok(())
    }

    fn on_three_clicks(&self) {
        println!("Three clicks detected. Starting action...");
        // Add code to start the desired action here
    }

    pub fn track_click(&self) {
        let click_count = CLICK_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

        if click_count >= 3 {
            self.on_three_clicks();
            CLICK_COUNT.store(0, Ordering::SeqCst); // Reset click count
        }
    }

    pub fn get_click_count(&self) -> u32 {
        CLICK_COUNT.load(Ordering::SeqCst)
    }

    pub fn is_pressed(&self) -> Result<bool, Box<dyn std::error::Error>> {
        self.0.is_pressed()
    }
}
