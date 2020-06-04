//
// Copyright (c) 2020 Nathan Fiedler
//
use riff::{Chunk, ChunkContents};
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        if let Err(err) = read_file(Path::new(&args[1])) {
            println!("error: {}", err);
        }
    } else {
        println!("Usage: riffinfo <filename>");
    }
}

fn read_file(filepath: &Path) -> io::Result<()> {
    let mut file = File::open(filepath)?;
    let chunk = Chunk::read(&mut file, 0)?;
    let contents = read_chunk(&chunk, &mut file);
    print_contents(0, &contents);
    Ok(())
}

fn read_chunk<T>(chunk: &Chunk, file: &mut T) -> ChunkContents
where
    T: std::io::Seek + std::io::Read,
{
    let id = chunk.id();
    if id == riff::RIFF_ID || id == riff::LIST_ID {
        let chunk_type = chunk.read_type(file).unwrap();
        let children = read_items(&mut chunk.iter(file));
        let mut children_contents: Vec<ChunkContents> = Vec::new();
        for child in children {
            children_contents.push(read_chunk(&child, file));
        }
        ChunkContents::Children(id, chunk_type, children_contents)
    } else if id == riff::SEQT_ID {
        let children = read_items(&mut chunk.iter_no_type(file));
        let mut children_contents: Vec<ChunkContents> = Vec::new();
        for child in children {
            children_contents.push(read_chunk(&child, file));
        }
        ChunkContents::ChildrenNoType(id, children_contents)
    } else {
        let contents = chunk.read_contents(file).unwrap();
        ChunkContents::Data(id, contents)
    }
}

fn read_items<T>(iter: &mut T) -> Vec<T::Item>
where
    T: Iterator,
{
    let mut vec: Vec<T::Item> = Vec::new();
    for item in iter {
        vec.push(item);
    }
    vec
}

fn print_contents(indent: u32, contents: &ChunkContents) {
    let mut padding = String::new();
    for _ in 0..indent {
        padding.push(' ');
    }
    match contents {
        ChunkContents::Data(id, data) => {
            println!("{}data -> id: {}, len: {}", padding, id, data.len());
        }
        ChunkContents::Children(id, typ, more) => {
            println!(
                "{}children -> id: {}, typ: {}, len: {}",
                padding,
                id,
                typ,
                more.len()
            );
            // skip over data sections that are not interesting
            if typ.as_str() != "movi" {
                for content in more.iter() {
                    print_contents(indent + 2, &content);
                }
            }
        }
        ChunkContents::ChildrenNoType(id, more) => {
            println!("{}no-type -> id: {}, len: {}", padding, id, more.len());
            for content in more.iter() {
                print_contents(indent + 2, &content);
            }
        }
    }
}
