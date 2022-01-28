use std::io::Write;
use std::fs;
use core::convert::TryInto;

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
struct Block { tokens: Vec<u8>, reference: Number, block_len: u8 }

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

fn process_tokens(tokens: Tokens) -> Vec<Block> {
    println!("Tokens = {:?}", tokens);

    let mut temp: Tokens = Vec::new();
    let mut result: Vec<Block> = Vec::new();

    // TODO: Improve
    // Split byte-stream in chunks and then
    // compress them.

    for token in tokens {
        temp.push(token);

        if temp.len() == 4 {
            result.push(process_chunk(&temp));
            temp.clear();
        }
    }
    result
}

fn dump_result(blocks: Vec<Block>) -> () {
    let mut f = fs::File::create("data/results.data").unwrap();

    blocks.iter().for_each(|b| {
        f.write(&b.reference.to_be_bytes()).unwrap();
        f.write(&[b.block_len]).unwrap();

        b.tokens.iter().for_each(|t| {
            f.write(&[*t]).unwrap();
        });
    });
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

    let result = process_tokens(tokens);

    dump_result(result)
}
