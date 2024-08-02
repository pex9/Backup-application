use std::thread;
use std::time::Duration;
use mouse_track::types::Point;
use mouse_track::Mouse;
use winit::event_loop;


fn main() {
    let event_loop = event_loop::EventLoop::new();
    let primary_monitor = event_loop.primary_monitor().unwrap();
    let size = primary_monitor.size();
    let h = size.height as i32;
    let w = size.width as i32;
  
    let mut mouse = Mouse::new();
    loop {

        if (mouse.get_position().unwrap() == Point{x:0,y:0}) {
            match mouse.rectangle_write(0, 0, w-1, h-1) {
                Ok(flag) => {
                    if flag == true {
                        thread::sleep(Duration::from_secs(2));
                        mouse.confirm().unwrap();
                    }
                }
                Err(_) => {}
            }
            break;
        }
    }
}
