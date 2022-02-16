use crate::{Block, Tokens, Number};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

pub struct ChunkCompressor {
    rx: Arc<Mutex<Receiver<Option<Tokens>>>>,
    tx: Sender<Option<Block>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

fn find_ref(tokens: &Tokens) -> &Number {
    tokens.iter().min().unwrap()
}

fn reduce(ref_: &Number, tokens: &Tokens) -> Tokens {
    tokens.iter().map(|t| *t - ref_).collect::<Tokens>()
}

fn get_bytes_needed(token: &u32) -> u8 {
    let mut base_mask: u32 = u32::from_be_bytes([0xFF, 0xFF, 0xFF, 0xFF]);

    let mut bytes = 0;

    while (token & base_mask) != 0 {
        // add one zero on the right side
        base_mask = base_mask << 8;
        bytes += 1;
    }

    return bytes;
}

fn compress(bytes_per_token: u8, tokens: &Tokens) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for t in tokens.iter() {
        let a = t.to_be_bytes().clone();

        for i in 0..bytes_per_token {
            let value = a[a.len() - bytes_per_token as usize + i as usize];
            result.push(value);
        }
    }

    result
}

fn process_chunk(tokens: &Tokens) -> Block {
    let ref_ = find_ref(&tokens);
    let reduced_t = reduce(&ref_, &tokens);

    // let's find out how many bytes
    // are needed for representing the max
    // value within tokens.
    let max_token = reduced_t.iter().max();
    let block_size = get_bytes_needed(max_token.unwrap());

    let compressed = compress(block_size, &reduced_t);

    Block { reference: *ref_, block_size, tokens: compressed }
}

impl ChunkCompressor {

    pub fn new(rx: Receiver<Option<Tokens>>, tx: Sender<Option<Block>>) -> ChunkCompressor {
        ChunkCompressor {
            thread: None, tx, rx: Arc::new(Mutex::new(rx))
        }
    }

    pub fn start(&mut self) {
        let rx = self.rx.clone();
        let tx = self.tx.clone();

        self.thread = Some(std::thread::spawn(move || {
            loop {
                let value = (*rx).lock().unwrap().recv().unwrap();

                if value.is_none() {
                    tx.send(None).unwrap();
                    break;
                }

                let result = process_chunk(&value.unwrap());
                tx.send(Some(result)).unwrap();
            }
        }));
    }

    pub fn stop(self) -> () {
        self.thread.unwrap().join().unwrap();
    }
}
