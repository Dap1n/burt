use std::{
    fs::OpenOptions,
    io::Write,
    panic,
    path::Path,
    sync::{Mutex, Once},
    thread,
};

static FILE_MUTEX: Mutex<()> = Mutex::new(());
static INIT: Once = Once::new();

pub fn setup_panic_logging(log_path: Box<Path>) {
    INIT.call_once(|| {
        panic::set_hook(Box::new(move |info| {
            let _guard = FILE_MUTEX.lock().unwrap();

            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic payload"
            };

            let location = info
                .location()
                .map(|loc| format!(" at '{}' line {}", loc.file(), loc.line()))
                .unwrap_or_default();

            let thread = thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            let message = format!(
                "thread '{}' panicked with '{}'{}\n",
                thread_name, payload, location,
            );

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path.as_ref())
            {
                let _ = file.write_all(message.as_bytes());
            }
        }));
    });
}
