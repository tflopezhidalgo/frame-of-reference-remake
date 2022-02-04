use crate::Block;

use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

pub struct Writer {
    rx: Arc<Mutex<Receiver<Option<Block>>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Writer {

    pub fn new(rx: Receiver<Option<Block>>) -> Writer {
        Writer { rx: Arc::new(Mutex::new(rx)), thread: None }
    }

    pub fn start(&mut self) -> () {
        let rx = self.rx.clone();

        self.thread = Some(std::thread::spawn(move || {
            let mut f = fs::File::create("data/results.data").unwrap();

            loop {
                let b = (*rx).lock().unwrap().recv().unwrap();

                if b.is_none() {
                    break;
                }

                f.write(&b.as_ref().unwrap().reference.to_be_bytes()).unwrap();
                f.write(&[b.as_ref().unwrap().block_len]).unwrap();

                b.unwrap().tokens.iter().for_each(|t| {
                    f.write(&[*t]).unwrap();
                });
            }
        }))
    }

    pub fn stop(self) -> () {
        match self.thread {
            Some(t) => t.join().unwrap(),
            _ => {}
         }
    }
}
