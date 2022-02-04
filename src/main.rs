mod writer;
mod compression;

use std::fs;
use std::thread;
use core::convert::TryInto;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

/*
 * Assumpsions
 *  - File is composed only by numbers separated by ','.
 *  - None of the numbers is higher than 255.
 *
 * We want to:
 *  - Dettach reading steps from writing steps (through a queue).
 *  - Do bulk reading.
 *  - Take a block of 4 numbers, compress them in a u8 vector and write them to file.
 */

pub type Number = u32;
pub type Tokens = Vec<Number>;
pub struct Block { tokens: Vec<u8>, reference: Number, block_len: u8 }

fn process_tokens(tokens: Tokens, q: Sender<Option<Tokens>>) -> () {
    println!("Tokens = {:?}", tokens);

    let mut temp: Tokens = Vec::new();

    // TODO: Improve
    // Split byte-stream in chunks and then
    // compress them.

    for token in tokens {
        temp.push(token);

        if temp.len() == 4 {
            q.send(Some(temp.clone())).unwrap();
            temp.clear();
        }
    }

    q.send(None).unwrap();
}

fn main() {
    let f_bytes: Vec<u8> = fs::read("data/numbers.data").unwrap();

    let mut tokens: Tokens = Vec::new();
    let mut temp: Vec<u8> = Vec::new();

    // TODO: improve

    for b in f_bytes {
        temp.push(b);

        if temp.len() == 4 {
            tokens.push(u32::from_be_bytes(temp.clone().try_into().unwrap()));
            temp.clear();
        }
    }

    let (chunk_tx, chunk_rx): (Sender<Option<Tokens>>, Receiver<Option<Tokens>>) = mpsc::channel();
    let (block_tx, block_rx): (Sender<Option<Block>>, Receiver<Option<Block>>) = mpsc::channel();

    process_tokens(tokens, chunk_tx);

    let mut writer = writer::Writer::new(block_rx);
    writer.start();

    let mut worker = compression::ChunkCompressor::new(chunk_rx, block_tx);
    worker.start();

    worker.stop();
    writer.stop();
}
