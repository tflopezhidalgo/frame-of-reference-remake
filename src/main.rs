mod writer;

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

type Number = u32;
type Tokens = Vec<Number>;
pub struct Block { tokens: Vec<u8>, reference: Number, block_len: u8 }

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

fn process_tokens(tokens: Tokens, q: Sender<Option<Block>>) -> () {
    println!("Tokens = {:?}", tokens);

    let mut temp: Tokens = Vec::new();

    // TODO: Improve
    // Split byte-stream in chunks and then
    // compress them.

    for token in tokens {
        temp.push(token);

        if temp.len() == 4 {
            let block = process_chunk(&temp);
            q.send(Some(block)).unwrap();
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

    let (tx, rx): (Sender<Option<Block>>, Receiver<Option<Block>>) = mpsc::channel();

    let mut writer = writer::Writer::new(rx);
    writer.start();

    let worker_thread = thread::spawn(move || {
        process_tokens(tokens, tx);
    });

    worker_thread.join().unwrap();
    writer.stop();
}
