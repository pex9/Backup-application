use crate::types::keys::Keys;
use crate::types::Point;
use std::error::Error;

use libc;

use libc::{c_int, c_void, c_char};
use std::ptr;

type XDO = *const c_void;
type WINDOW = c_int;
type INTPTR = *mut c_int;

fn xdo_translate_key(key: &Keys) -> c_int {
    match key {
        Keys::LEFT => 1,
        Keys::WHEEL | Keys::MIDDLE => 2,
        Keys::RIGHT => 3,
        _ => panic!("Invalid key passed: {:?}", key)
    }
}

impl From<(c_int, c_int)> for Point {
    fn from(other: (c_int, c_int)) -> Point {
        Point {
            x: other.0 as _,
            y: other.1 as _,
        }
    }
}

pub struct Mouse {
    xdo: XDO,
    current_window: c_int
}

#[link(name = "xdo")]
extern "C" {
    fn xdo_new(display: *const c_char) -> XDO;
    fn xdo_free(xdo: XDO);
    fn xdo_get_mouse_location(xdo: XDO, x: INTPTR, y: INTPTR, screen_num: INTPTR);
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            xdo: unsafe { xdo_new(ptr::null()) },
            current_window: 0
        }
    }

    pub fn get_position(&self) -> Result<Point, Box<dyn Error>> {
        let pos: Point;
        unsafe {
            let mut x: c_int = 0;
            let mut y: c_int = 0;
            let mut _screen_num: c_int = 0;
            xdo_get_mouse_location(self.xdo, &mut x as INTPTR, &mut y as INTPTR, &mut _screen_num as INTPTR);
            pos = (x, y).into();
        }

        Ok(pos)
    }
}

impl Drop for Mouse {
    fn drop(&mut self) {
        unsafe { 
            xdo_free(self.xdo); 
        }
    }
}
