use std::sync::Arc;

use crate::types::keys::Keys;
use crate::{sys, types::Point};

pub struct Rectangle<'a> {
    x: i32,
    y: i32,
    mouse: Arc<&'a mut Mouse>,
    width: i32,
    height: i32,
}

impl<'a> Rectangle<'a> {
    pub fn new(mouse: Arc<&'a mut Mouse>, width: i32, height: i32) -> Self {
        Rectangle {
            x: mouse.get_position().unwrap().x,
            y: mouse.get_position().unwrap().y,
            mouse,
            width,
            height
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn draw_rectangle(&mut self) -> bool {
        let tol = 20;
        let mut flag = true;
        
        // Left -> Right
        while self.mouse.get_position().unwrap().x < self.x + self.width && flag==true {
            match self.mouse.get_position().unwrap().y < self.y + tol && self.mouse.get_position().unwrap().y > self.y - tol {
                true => {},
                false => flag = false 
            }
        }
        self.set_position(self.x + self.width, self.y);

        // Up -> Down
        while self.mouse.get_position().unwrap().y < self.y + self.height && flag==true{
            match self.mouse.get_position().unwrap().x < self.x + tol && self.mouse.get_position().unwrap().x > self.x - tol {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x, self.y + self.height);
        
        // Right -> Left
        while self.mouse.get_position().unwrap().x > self.x-self.width && flag==true {
            match self.mouse.get_position().unwrap().y < self.y + tol && self.mouse.get_position().unwrap().y > self.y - tol {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x-self.width, self.y);

        // Down -> Up
        while self.mouse.get_position().unwrap().y > self.y-self.height && flag==true {
            match self.mouse.get_position().unwrap().x < self.x + tol && self.mouse.get_position().unwrap().x > self.x - tol {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x, self.y-self.height);

        flag
    }
}

impl<'a> Drop for Rectangle<'a> {
    fn drop(&mut self) {}
}

pub struct Mouse(sys::Mouse);

#[allow(unreachable_code, unused_variables)]
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
        self.0.wheel(delta)
    }

    // Press and release the button in one function
    pub fn click<'a>(&self, button: &'a Keys) -> Result<(), Box<dyn std::error::Error + 'a>> {
        self.0.press(button).unwrap_or(());
        self.0.release(button)
    }

    // Wrapper to verify the rectangle is drawn, then we can start the backup
    pub fn rectangle_write(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
        let mut rect = Rectangle::new(Arc::clone(&Arc::new(self)), width, height);
        match rect.draw_rectangle() {
            true => println!("Rectangle drawn"),                        // In final version, we will start the backup here
            false => println!("We weren't drawing the rectangle")       // In final version, we will do nothing here
        }
        Ok(())
    }
}