use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use rglk::{
    entry::{GlkMessage, GlkResult},
    prelude::*,
};

#[derive(Debug, Default)]
pub struct SimpleWindow {
    winid: GlkWindowID,
    request: Option<Receiver<GlkMessage>>,
    result: Option<Sender<GlkResult>>,
}

impl GlkWindow for SimpleWindow {
    fn new(request: Receiver<GlkMessage>, result: Sender<GlkResult>) -> Self {
        Self {
            request: Some(request),
            result: Some(result),
            winid: 0,
        }
    }

    fn run(&mut self) {
        while let Ok(message) = self.request.as_ref().unwrap().recv() {
            match message {
                GlkMessage::Write { winid, message } => {
                    println!("[Window {winid}]: {message}");
                    let _ = self
                        .result
                        .as_ref()
                        .unwrap()
                        .send(GlkResult::Result(message.len()));
                }
                GlkMessage::Open(winid) => println!("[OPEN window {winid}]"),
                GlkMessage::Split { parent, winid, .. } => {
                    println!("[SPLIT parent {parent} -> {winid}]");
                }
            }
        }
    }

    fn get_size(&self) -> GlkWindowSize {
        todo!()
    }

    fn move_cursor(&mut self, _x: u32, _y: u32) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn get_line(&mut self, event: LineInput, _initlen: usize, tx: Sender<GlkEvent>) {
        let win = self.winid;
        println!("get line from {win}");
        let _ = thread::spawn(move || {
            let is_latin1 = match event {
                LineInput::Latin1(val) => {
                    println!(
                        "{}",
                        val.iter().map(|byte| *byte as char).collect::<String>()
                    );
                    true
                }
                LineInput::Unicode(val) => {
                    println!(
                        "{}",
                        val.iter()
                            .map(|long| char::from_u32(*long).unwrap())
                            .collect::<String>()
                    );
                    false
                }
            };
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line); // <- convert to actual readline
            println!(">>> read '{line}' <<<");
            let _ = tx.send(if is_latin1 {
                GlkEvent::LineInput {
                    win,
                    buf: LineInput::Latin1(line.into()),
                }
            } else {
                GlkEvent::LineInput {
                    win,
                    buf: LineInput::Unicode(line.chars().map(|ch| ch as u32).collect::<Vec<_>>()),
                }
            });
        });
    }
}
