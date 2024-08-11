use utils::perform_backup;

mod backup;
mod config;
mod config_gui;
mod confirm_gui;
mod utils;

/* run the confirmation gui
fn main() {
    let (sender, receiver) = std::sync::mpsc::channel();

    thread::spawn(move||{
        match receiver.recv() {
            Ok(choice) => {
                match choice {
                    Choice::Yes => {
                        print!("1");
                        std::process::exit(0);
                    }
                    Choice::No => {
                        print!("2");
                        std::process::exit(0);
                    }
                }
            }
            Err(_) => {
                print!("3");
                std::process::exit(0);
            }
        }
    });

    run_confirm_gui(sender);

}
 */
/*
// run the configuration gui
fn main(){
    run_config_gui().unwrap();
}
*/
// use this if want to activate the program when load pc (windows tested, need to check also linux,macos)
/*fn main()
{
    auto_launch_app("/path/to/app");
}
*/
/* //use to run the back up function and write the corrispond log at the end of the operation

*/
fn main() {
    perform_backup().unwrap();
}
