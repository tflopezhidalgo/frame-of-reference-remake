use crate::Block;

use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

static OUTPUT_FILE: &str = "data/results.data";

pub struct Writer {
    rxs: Vec<Arc<Mutex<Receiver<Option<Block>>>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Writer {

    pub fn new(rx: Vec<Receiver<Option<Block>>>) -> Writer {
        let mut _rx = Vec::new();

        for x in rx {
            _rx.push(Arc::new(Mutex::new(x)));
        }

        Writer { rxs: _rx, thread: None }
    }

    pub fn start(&mut self) -> () {
        let rxs = self.rxs.clone();

        self.thread = Some(std::thread::spawn(move || {
            let mut f = fs::File::create(OUTPUT_FILE).unwrap();
            let mut skipped_queues: Vec<u32> = Vec::new();

            loop {
                if skipped_queues.len() == 2 {
                    break;
                }

                let mut current_idx = 0;

                for rx in &rxs {
                    if skipped_queues.contains(&current_idx) {
                        current_idx = current_idx + 1;
                        continue;
                    }

                    let b = (*rx).lock().unwrap().recv().unwrap();

                    if b.is_none() {
                        // TODO index
                        skipped_queues.push(current_idx);
                        current_idx = current_idx + 1;
                        continue;
                    }

                    f.write(&b.as_ref().unwrap().reference.to_be_bytes()).unwrap();
                    f.write(&[b.as_ref().unwrap().block_len]).unwrap();

                    b.unwrap().tokens.iter().for_each(|t| {
                        f.write(&[*t]).unwrap();
                    });

                    current_idx = current_idx + 1;
                }
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
