mod compressor;
mod writer;

use core::convert::TryInto;
use std::fs;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

static INPUT_FILE: &str = "data/numbers.data";
static CONCURRENCY: u8 = 2;
static CHUNK_SIZE: usize = 4;
static U32_SIZE: usize = 4;

/*
 * We want to:
 *  - Dettach reading steps from writing steps (through a queue).
 *  - Do bulk reading.
 *  - Take a block of 4 numbers, compress it in a u8 vector and write it to file.
 *
 *  TBD:
 *  Idea: try to build 4-byte blocks and if it won't work
 *  start to iterate (size --) over block sizes until we find
 *  the one which gives us an entire number of chunks.
 */

pub type Number = u32;
pub type RawNumbers = Vec<Number>;

// Represents a set of raw numbers which
// will be compressed.
pub type Chunk = Vec<Number>;

pub type Tokens = Vec<u8>;

// Represents a compressed set of Numbers
// Chunks -> Block
#[derive(Debug)]
pub struct Block {
    tokens: Tokens,
    reference: Number,
    block_size: u8,
}

fn spread_chunks(raw_numbers: RawNumbers, worker_queues: Vec<Sender<Option<Chunk>>>) -> () {
    let mut current_idx = 0;

    for c in raw_numbers.chunks(CHUNK_SIZE) {
        let worker_idx = current_idx % worker_queues.len();
        worker_queues[worker_idx]
            .send(Some(c.to_vec()))
            .unwrap();

        println!("queued chunk = {:?} in worker {:?}", c, worker_idx);

        current_idx += 1;
    }

    for q in worker_queues {
        q.send(None).unwrap();
    }
}

fn main() {
    let f_bytes: Vec<u8> = fs::read(INPUT_FILE).unwrap();

    let mut raw_numbers: RawNumbers = Vec::new();

    // Build u32 numbers by grouping u8 ones in 4-sized groups (4 bytes).
    // We're assuming 32-bits-size numbers
    f_bytes
        .chunks(U32_SIZE)
        .for_each(|u32_number| raw_numbers.push(u32::from_be_bytes(u32_number.try_into().unwrap())));

    let mut compressors: Vec<compressor::Compressor> = Vec::new();
    let mut chunks_txs: Vec<Sender<Option<Chunk>>> = Vec::new();
    let mut block_rxs: Vec<Receiver<Option<Block>>> = Vec::new();

    for _ in 0..CONCURRENCY {
        let (chunk_tx, chunk_rx): (Sender<Option<Chunk>>, Receiver<Option<Chunk>>) =
            mpsc::channel();

        // we want to keep each worker with its own queue.
        let (block_tx, block_rx): (Sender<Option<Block>>, Receiver<Option<Block>>) =
            mpsc::channel();

        /*
         * Set to each compressor a channel for:
         *  - Receiving incoming chunks
         *  - Queueing after-compression blocks
         */
        let mut c = compressor::Compressor::new(chunk_rx, block_tx);
        c.start();

        chunks_txs.push(chunk_tx);
        compressors.push(c);
        block_rxs.push(block_rx);
    }
    let mut writer = writer::Writer::new(block_rxs);
    writer.start();

    spread_chunks(raw_numbers, chunks_txs);

    for c in compressors {
        c.stop();
    }

    writer.stop();
}
