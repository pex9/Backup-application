use std::{error, fmt};

use core_graphics::{
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};

use crate::types::Point;

impl From<CGPoint> for Point {
    fn from(other: CGPoint) -> Point {
        Point {
            x: other.x as _,
            y: other.y as _,
        }
    }
}

impl Into<CGPoint> for Point {
    fn into(self) -> CGPoint {
        CGPoint::new(self.x as _, self.y as _)
    }
}

#[derive(Debug)]
pub enum Error {
    CGEventNotCreated,
    CGEventSourceStateInvalid,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CGEventNotCreated => write!(f, "CGEvent could not be created"),
            Error::CGEventSourceStateInvalid => write!(f, "invalid CGEventSourceStateID"),
        }
    }
}

pub struct Mouse;

impl Mouse {
    fn event_source() -> Result<CGEventSource, Error> {
        Ok(
            CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
                .or(Err(Error::CGEventSourceStateInvalid))?,
        )
    }

    pub fn new() -> Mouse {
        Mouse
    }

    pub fn get_position(&self) -> Result<Point, Box<dyn error::Error>> {
        Ok(CGEvent::new(Self::event_source()?)
            .or(Err(Error::CGEventNotCreated))?
            .location()
            .into())
    }
}
