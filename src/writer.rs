use crate::Block;

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct Writer {
    // Channels where blocks to be written are going to come from
    rx_channels: Vec<Arc<Mutex<Receiver<Option<Block>>>>>,

    thread: Option<std::thread::JoinHandle<()>>,
}

impl<'a> Writer {
    const OUTPUT_FILE: &'a str = "data/results.data";

    pub fn new(rx_channels: Vec<Receiver<Option<Block>>>) -> Writer {
        let mut _rx_channels = Vec::new();

        for ch in rx_channels {
            _rx_channels.push(Arc::new(Mutex::new(ch)));
        }

        Writer {
            rx_channels: _rx_channels,
            thread: None,
        }
    }

    pub fn dump_block_into_file(block: Block, file: &mut fs::File) -> () {
        file.write(&block.reference.to_be_bytes()).unwrap();
        file.write(&[block.block_size]).unwrap();

        // We're implicity using big-endian
        block.tokens.iter().for_each(|t| {
            file.write(&[*t]).unwrap();
        });
    }

    pub fn start(&mut self) -> () {
        let rx_channels = self.rx_channels.clone();

        self.thread = Some(std::thread::spawn(move || {
            let mut f = fs::File::create(Writer::OUTPUT_FILE).unwrap();
            let mut skipped_idxs: HashSet<usize> = HashSet::new();

            // Numbers within a chunk. Let's keep it in 4
            f.write(&[0x04]).unwrap();

            while skipped_idxs.len() != rx_channels.len() {
                for i in 0..rx_channels.len() {
                    if skipped_idxs.contains(&i) {
                        // Channel was closed.
                        continue;
                    }

                    let ch = &rx_channels[i];

                    if let Some(b) = (*ch).lock().unwrap().recv().unwrap() {
                        Writer::dump_block_into_file(b, &mut f);
                    } else {
                        // Channel was closed.
                        skipped_idxs.insert(i);
                    }
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
