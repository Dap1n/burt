use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    sync::RwLock,
};

use crate::commands::tas::{
    HeldButtons,
    parser::{Point2D, TasScript},
};

pub(super) static RAW_LOGGER: RwLock<Option<RawLogger>> = RwLock::new(None);

pub(super) struct RawLogger {
    w: BufWriter<File>,
    last_written_tick: u32,
    current_tick: u32,
    current_movement: Point2D,
    current_angles: Point2D,
    current_buttons: HeldButtons,
}

impl RawLogger {
    pub fn new(path: impl AsRef<Path>, parsed_tas: &TasScript) -> io::Result<Self> {
        let f = File::create(path)?;
        let mut w = BufWriter::with_capacity(16 * 1024, f);
        writeln!(w, "version {}", parsed_tas.version)?;
        writeln!(w, "start {}", parsed_tas.start)?;
        Ok(Self {
            w,
            last_written_tick: 0,
            current_tick: 0,
            current_movement: Point2D::default(),
            current_angles: Point2D::default(),
            current_buttons: HeldButtons(0),
        })
    }

    pub fn tick(
        &mut self,
        tick: u32,
        movement: Point2D,
        angles: Point2D,
        buttons: HeldButtons,
        commands: Option<Box<str>>,
    ) -> io::Result<()> {
        self.current_tick = tick;
        if self.current_movement == movement
            && self.current_angles == angles
            && self.current_buttons == buttons
            && commands.is_none()
        {
            return Ok(());
        }
        write!(self.w, "{tick}>")?;
        if self.current_movement == movement {
            write!(self.w, "|")?;
        } else {
            self.current_movement = movement;
            write!(self.w, "{} {}|", movement.x, movement.y)?;
        }
        if self.current_angles == angles {
            write!(self.w, "|")?;
        } else {
            self.current_angles = angles;
            write!(self.w, "{} {}|", angles.x, angles.y)?;
        }
        if self.current_buttons != buttons {
            self.current_buttons = buttons;
            write!(self.w, "{}", buttons)?;
        }
        if let Some(commands) = commands {
            writeln!(self.w, "|{}", commands)?;
        } else {
            writeln!(self.w)?;
        }
        self.last_written_tick = tick;
        Ok(())
    }
}

impl Drop for RawLogger {
    fn drop(&mut self) {
        if self.current_tick > self.last_written_tick {
            let _ = writeln!(self.w, "{}>", self.current_tick);
        }
        let _ = self.w.flush();
    }
}
