use crate::{Block, Tokens, Number};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use core::convert::TryInto;

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

fn compress(tokens: &Tokens) -> Vec<u8> {
    tokens.iter().map(|t| { (*t).try_into().unwrap() }).collect::<Vec<u8>>()
}

fn process_chunk(tokens: &Tokens) -> Block {
    let ref_ = find_ref(&tokens);
    let reduced_t = reduce(&ref_, &tokens);

    let compressed = compress(&reduced_t);

    println!("Compressed = {:?}", compressed);
    println!("Reference = {:?}", ref_);

    // For the moment we support only up to 4 bytes.
    Block { reference: *ref_, block_len: 4, tokens: compressed }
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
