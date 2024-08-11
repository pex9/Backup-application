use std::sync::Arc;

use crate::types::keys::Keys;
use crate::{sys, types::Point};

const TOL: i32 = 50;
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Positive, 
    Negative
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Direction::Positive => match other {
                Direction::Positive => true,
                Direction::Negative => false
            },
            Direction::Negative => match other {
                Direction::Negative => true,
                Direction::Positive => false
            }
        }
    }
}

impl Eq for Direction {}

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
        let mut flag = true;
        
        // Left -> Right
        while self.mouse.get_position().unwrap().x < self.x + self.width && flag==true {
            match self.mouse.get_position().unwrap().y < self.y + TOL && self.mouse.get_position().unwrap().y > self.y - TOL {
                true => {},
                false => flag = false 
            }
        }
        self.set_position(self.x + self.width, self.y);

        // Up -> Down
        while self.mouse.get_position().unwrap().y < self.y + self.height && flag==true{
            match self.mouse.get_position().unwrap().x < self.x + TOL && self.mouse.get_position().unwrap().x > self.x - TOL {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x, self.y + self.height);
        
        // Right -> Left
        while self.mouse.get_position().unwrap().x > self.x-self.width && flag==true {
            match self.mouse.get_position().unwrap().y < self.y + TOL && self.mouse.get_position().unwrap().y > self.y - TOL {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x-self.width, self.y);

        // Down -> Up
        while self.mouse.get_position().unwrap().y > self.y-self.height && flag==true {
            match self.mouse.get_position().unwrap().x < self.x + TOL && self.mouse.get_position().unwrap().x > self.x - TOL {
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

pub struct Confirm<'a> {
    mouse: Arc<&'a mut Mouse>,
}

impl<'a> Confirm<'a> {
    pub fn new(mouse: Arc<&'a mut Mouse>) -> Self {
        Confirm {
            mouse
        }
    }


    pub fn confirm(&mut self) -> bool {
        let mut prec = self.mouse.get_position().unwrap();
        let mut history = Vec::<Direction>::new();
        let mut last: Option<Direction> = None;
        loop {
            let pos = self.mouse.get_position().unwrap();
            // if pos.x-prec.x>=0 {
            if pos.x-prec.x>=0 && pos.x+prec.y-pos.y-prec.x < TOL && pos.x+prec.y-pos.y-prec.x > -TOL {
                // println!("{} - {}: {}", pos.y, prec.y, pos.y-prec.y);       // Debugging
                // Positive
                if pos.y-prec.y<0 {
                    match last {
                        Some(dir) => match dir {
                            Direction::Positive => {
                            },
                            Direction::Negative => {
                                history.push(Direction::Positive);
                                last = Option::from(Direction::Positive);
                            }
                        }
                        None => {
                            history.push(Direction::Positive);
                            last = Option::from(Direction::Positive);
                        }
                    }
                }
                else if pos.y-prec.y>0 {
                    match last {
                        Some(dir) => match dir {
                            Direction::Negative => {},
                            Direction::Positive => {
                                history.push(Direction::Negative);
                                last = Option::from(Direction::Negative);
                            }
                        }
                        None => {
                            history.push(Direction::Negative);
                            last = Option::from(Direction::Negative);
                        }
                    }
                }
            } 
            else {
                last = None;
                history.clear();
            }

            if history.len() == 2 {
                if history[0] == Direction::Positive && history[1] == Direction::Negative {
                    return true;
                }
                else if history[0] == Direction::Negative && history[1] == Direction::Positive {
                    return false;
                }
                /*else {
                    last = None;
                    history.clear();
                }*/
            }
            prec = pos;
        }
    }  
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
    pub fn rectangle_write(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<bool, Box<dyn std::error::Error>> {
        let data =  Arc::new(self);
        let mut rect = Rectangle::new(Arc::clone(&data), width, height);
        let res = rect.draw_rectangle();
        match res {
            true => println!("Rectangle drawn"),                        // In final version, we will start the backup here
            false => println!("We weren't drawing the rectangle")       // In final version, we will do nothing here
        }
        Ok(res)
    }

    pub fn confirm(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(self);
        let mut conf = Confirm::new(Arc::clone(&data));
        match conf.confirm() {
            true => println!("Confirmed"),                              // In final version, we will confirm the backup here gui to insert and activate backup
            false => println!("We weren't confirming")                  // In final version, we will cancel the backup here
        }
        Ok(())
    }
}
