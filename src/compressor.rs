use crate::{Block, Chunk, Number, Tokens};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Compressor {
    rx: Arc<Mutex<Receiver<Option<Chunk>>>>,
    tx: Sender<Option<Block>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Compressor {
    pub fn new(rx: Receiver<Option<Chunk>>, tx: Sender<Option<Block>>) -> Self {
        Self {
            thread: None,
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    fn find_ref(c: &Chunk) -> &Number {
        c.iter().min().unwrap()
    }

    fn reduce(ref_: &Number, c: &Chunk) -> Chunk {
        c.iter().map(|t| *t - ref_).collect::<Chunk>()
    }

    fn get_bytes_needed(n: &Number) -> u8 {
        let mut base_mask: u32 = u32::from_be_bytes([0xFF, 0xFF, 0xFF, 0xFF]);

        let mut bytes = 0;

        while (n & base_mask) != 0 {
            // add one zero on the right side
            base_mask = base_mask << 8;
            bytes += 1;
        }

        return bytes;
    }

    fn compress(bytes_per_token: u8, c: &Chunk) -> Tokens {
        let mut result: Vec<u8> = Vec::new();

        for t in c.iter() {
            let a = t.to_be_bytes().clone();

            for i in 0..bytes_per_token {
                let value = a[a.len() - bytes_per_token as usize + i as usize];
                result.push(value);
            }
        }

        result
    }

    fn process_chunk(c: &Chunk) -> Block {
        let ref_ = Self::find_ref(&c);
        let reduced_t = Self::reduce(&ref_, &c);

        // let's find out how many bytes
        // are needed for representing the max
        // value within tokens.
        let max_token = reduced_t.iter().max();
        let block_size = Self::get_bytes_needed(max_token.unwrap());

        let compressed = Self::compress(block_size, &reduced_t);

        Block {
            reference: *ref_,
            block_size,
            tokens: compressed,
        }
    }

    pub fn start(&mut self) {
        let rx = self.rx.clone();
        let tx = self.tx.clone();

        self.thread = Some(std::thread::spawn(move || loop {
            let value = (*rx).lock().unwrap().recv().unwrap();

            if value.is_none() {
                tx.send(None).unwrap();
                break;
            }

            let result = Self::process_chunk(&value.unwrap());
            tx.send(Some(result)).unwrap();
        }));
    }

    pub fn stop(self) -> () {
        self.thread.unwrap().join().unwrap();
    }
}
