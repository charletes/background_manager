use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

mod ui;

#[derive(Debug, Clone)]
enum DaemonAction {
    Nothing,
    Quit,
}

fn main() -> Result<(), slint::PlatformError> {
    let (tx, rx) = mpsc::channel();
    let daemon_handle = start_daemon_thread(rx);
    ui::run()?;

    // Send quit event after UI closes
    tx.send(DaemonAction::Quit).unwrap();
    println!("Sent Quit event to daemon thread");

    daemon_handle.join().unwrap();
    Ok(())
}

fn start_daemon_thread(rx: Receiver<DaemonAction>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Check the queue with a timeout
            match rx.recv_timeout(Duration::from_millis(1000)) {
                Ok(action) => {
                    println!("Received event: {:?}", action);
                    match action {
                        DaemonAction::Quit => {
                            println!("Daemon thread exiting...");
                            break;
                        }
                        DaemonAction::Nothing => {
                            println!("Nothing action received, continuing...");
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    println!("Timeout expired, no event received");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("Channel disconnected, daemon thread exiting...");
                    break;
                }
            }
        }
    })
}
