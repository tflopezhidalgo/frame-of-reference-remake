mod writer;
mod compression;

use std::fs;
use core::convert::TryInto;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

static INPUT_FILE: &str = "data/numbers.data";
static CONCURRENCY: u8 = 2;

/*
 * We want to:
 *  - Dettach reading steps from writing steps (through a queue).
 *  - Do bulk reading.
 *  - Take a block of 4 numbers, compress them in a u8 vector and write them to file.
 *
 *  TBD:
 *  Idea: try to build 4-byte blocks and if it won't work
 *  start to iterate (size --) over block sizes until we find
 *  the one which gives us an entire number of chunks.
 */

pub type Number = u32;
pub type Tokens = Vec<Number>;


// Represents a compressed set of Numbers
#[derive(Debug)]
pub struct Block { tokens: Vec<u8>, reference: Number, block_size: u8 }


fn queue_chunks(tokens: Tokens, worker_queues: Vec<Sender<Option<Tokens>>>) -> () {
    let mut temp: Tokens = Vec::new();

    // TODO: Improve
    // Split byte-stream in chunks and compress them.

    let mut current_idx = 0;

    for token in tokens {
        temp.push(token);

        if temp.len() == 4 {
            let worker_idx = current_idx % worker_queues.len();
            worker_queues[worker_idx].send(Some(temp.clone())).unwrap();

            println!("Queued chunk = {:?} in worker {:?}", temp, worker_idx);

            current_idx += 1;
            temp.clear();
        }
    }

    for q in worker_queues {
        q.send(None).unwrap();
    }
}


fn main() {
    let f_bytes: Vec<u8> = fs::read(INPUT_FILE).unwrap();

    let mut tokens: Tokens = Vec::new();
    let mut temp: Vec<u8> = Vec::new();

    // Build u32 numbers by grouping u8 ones
    // in groups of size 4 (4 bytes).

    // TODO: improve
    for b in f_bytes {
        temp.push(b);

        if temp.len() == 4 {
            tokens.push(u32::from_be_bytes(temp.clone().try_into().unwrap()));
            temp.clear();
        }
    }

    let mut compressors: Vec<compression::ChunkCompressor> = Vec::new();
    let mut work_queues: Vec<Sender<Option<Tokens>>> = Vec::new();
    let mut result_queues: Vec<Receiver<Option<Block>>> = Vec::new();

    for _ in 0..CONCURRENCY {
        let (chunk_tx, chunk_rx): (Sender<Option<Tokens>>, Receiver<Option<Tokens>>) = mpsc::channel();

        // we want to keep each worker with its own queue.
        let (block_tx, block_rx): (Sender<Option<Block>>, Receiver<Option<Block>>) = mpsc::channel();

        let mut worker = compression::ChunkCompressor::new(chunk_rx, block_tx);
        worker.start();

        work_queues.push(chunk_tx);
        compressors.push(worker);
        result_queues.push(block_rx);
    }
    let mut writer = writer::Writer::new(result_queues);
    writer.start();

    queue_chunks(tokens, work_queues);

    for c in compressors {
        c.stop();
    }

    writer.stop();
}
