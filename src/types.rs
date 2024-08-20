use std::{fmt, thread};
use std::sync::{Arc, Mutex};

use crate::mouse::Mouse;

pub mod keys;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Point {}

const TOL: i32 = 60;
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
            thread::sleep(std::time::Duration::from_millis(100));
            match self.mouse.get_position().unwrap().y < self.y + TOL && self.mouse.get_position().unwrap().y > self.y - TOL {
                true => {},
                false => flag = false 
            }
        }
        self.set_position(self.x + self.width, self.y);

        // Up -> Down
        while self.mouse.get_position().unwrap().y < self.y + self.height && flag==true{
            thread::sleep(std::time::Duration::from_millis(100));
            match self.mouse.get_position().unwrap().x < self.x + TOL && self.mouse.get_position().unwrap().x > self.x - TOL {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x, self.y + self.height);
        
        // Right -> Left
        while self.mouse.get_position().unwrap().x > self.x-self.width && flag==true {
            thread::sleep(std::time::Duration::from_millis(100));
            match self.mouse.get_position().unwrap().y < self.y + TOL && self.mouse.get_position().unwrap().y > self.y - TOL {
                true => {},
                false => flag = false
            }
        }
        self.set_position(self.x-self.width, self.y);

        // Down -> Up
        while self.mouse.get_position().unwrap().y > self.y-self.height && flag==true {
            thread::sleep(std::time::Duration::from_millis(100));
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
    init: Option<Point>
}

impl<'a> Confirm<'a> {
    pub fn new(mouse: Arc<&'a mut Mouse>) -> Self {
        Confirm {
            mouse,
            init: None
        }
    }


    pub fn confirm(&mut self,controller: Arc<Mutex<bool>>) -> bool {
        let mut prec = self.mouse.get_position().unwrap();
        let mut history = Vec::<Direction>::new();
        let mut last: Option<Direction> = None;
        loop {
            let lk = controller.lock().unwrap();
            // If backup is already in progress, exit early
            if *lk==true {
                return false;
            }
            drop(lk);

            thread::sleep(std::time::Duration::from_millis(20));


            let pos = self.mouse.get_position().unwrap();
            // Testing
            println!("Prec: {:?}", prec);
            println!("Actual: {:?}", pos);
            println!("Diff: {:?}", pos.x+prec.y-pos.y-prec.x);
            println!("History: {:?}", history);

            if pos.x-prec.x>=0 && pos.x+prec.y-pos.y-prec.x < TOL && pos.x+prec.y-pos.y-prec.x > -TOL {
                if last == None {
                    self.init = Option::from(prec.clone());
                }

                // Positive
                if pos.y-prec.y<0 {
                    match last {
                        Some(dir) => match dir {
                            Direction::Positive => {
                            },
                            Direction::Negative => {
                                if self.init.unwrap().x + TOL <= pos.x {
                                    history.push(Direction::Positive);
                                    last = Option::from(Direction::Positive);
                                }
                                else {
                                    last = None;
                                    self.init = None;
                                    history.clear();
                                }
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
                                if self.init.unwrap().x + TOL <= pos.x {
                                    history.push(Direction::Negative);
                                    last = Option::from(Direction::Negative);
                                }
                                else {
                                    last = None;
                                    self.init = None;
                                    history.clear();
                                }
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
                self.init = None;
                history.clear();
            } 
            
            if history.len() == 2 {
                if history[0] == Direction::Positive && history[1] == Direction::Negative {
                    if self.init.unwrap().y <= pos.y  { 
                        return true;
                    }
                }
                else if history[0] == Direction::Negative && history[1] == Direction::Positive {
                    if self.init.unwrap().y >= pos.y {
                        return false;
                    }
                }
                else {
                    last = None;
                    self.init = None;
                    history.clear();
                }
            }
            

            prec = pos;
        }
    }  
}

