use std::path::Path;

use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt};
use chroma_dbg::ChromaConfig;
use gwynn_pyc::obj::{Object, ObjectReader};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("No .pyc file specified");
    let mut file = std::fs::File::open(&path)?;

    let chroma = ChromaConfig {
        inline_array: chroma_dbg::InlineThreshold::Always,
        integer_format: chroma_dbg::IntegerFormat::AlwaysHex,
        ..ChromaConfig::DEFAULT
    };
    let version = file.read_u16::<LittleEndian>()?;
    let _ = file.read_u16::<LittleEndian>()?; // magic header \n\r
    let _pad = file.read_u32::<LittleEndian>()?;
    let mut reader = ObjectReader::new(file);
    let obj = gwynn_pyc::obj::read_obj(&mut reader).context("Failed to read .pyc file")?;
    println!("Python version: {version}");
    // println!("{:#?}", obj);
    print_obj(&reader.refs, &obj, 0);

    // if let Object::Code { filename, .. } = obj {
    //     println!("Filename: {:?}", filename.to_string());
    //     let Some(filename) = filename.to_string() else {
    //         return Ok(());
    //     };
    //     let filename = Path::new(&filename);
    //     if filename.has_root() {
    //         println!("Filename is absolute, not copying");
    //         return Ok(());
    //     }
    //     let destination = Path::new("pyc_dump").join(filename);
    //     if let Some(parent) = destination.parent() {
    //         std::fs::create_dir_all(parent)?;
    //     }
    //     std::fs::copy(&path, &destination)
    //         .with_context(|| format!("Failed to copy {path:?} to {destination:?}"))?;
    // }

    Ok(())
}

fn print_obj(refs: &[Object], obj: &Object, indent: usize) {
    let indent_str = " ".repeat(indent);
    match obj {
        Object::Code {
            argcount,
            posonlyargcount,
            kwonlyargcount,
            stacksize,
            flags,
            code,
            constants,
            names,
            localsplusnames,
            localspluskinds,
            filename,
            name,
            qualname,
            firstlineno,
            linetable,
            exceptiontable,
        } => {
            println!(
                "{indent_str}Code Object: {name} (argcount: {argcount}, posonlyargcount: {posonlyargcount}, kwonlyargcount: {kwonlyargcount}, stacksize: {stacksize}, flags: {flags:#X}, firstlineno: {firstlineno})"
            );
            println!("{indent_str}Filename: {filename:?}");
            println!("{indent_str}Bytecode:");
            print_obj(refs, code, indent + 1);
            println!("{indent_str}constants:");
            print_obj(refs, constants, indent + 1);
            println!("{indent_str}localsplusnames:");
            print_obj(refs, localsplusnames, indent + 1);
            println!("{indent_str}localspluskinds:");
            print_obj(refs, localspluskinds, indent + 1);
            println!("{indent_str}names:");
            print_obj(refs, names, indent + 1);
            // println!("{indent_str}qualname:");
            // print_obj(refs, qualname, indent + 1);
        }
        Object::Tuple(objects) => {
            println!("{indent_str}Tuple (len={}):", objects.len());
            for o in objects {
                print_obj(refs, o, indent + 1);
            }
        }
        Object::List(objects) => todo!(),
        Object::Set(objects) => todo!(),
        Object::Ref(r) => {
            println!("{indent_str}Ref: {r}");
            if let Some(o) = refs.get(*r as usize) {
                print_obj(refs, o, indent + 1);
            } else {
                println!("{indent_str}  Invalid ref");
            }
        }
        Object::None => println!("{indent_str}None"),
        Object::Null => todo!(),
        Object::Ellipsis => todo!(),
        Object::Bool(b) => {
            println!("{indent_str}Bool: {b}");
        }
        Object::ByteString(data) => {
            if data.len() > 4096 {
                println!(
                    "{indent_str}ByteString (len={}) {}...",
                    data.len(),
                    hex::encode(&data[0..16])
                );
                std::fs::write("pyc_dump/bytes_large.bin", data);
                return;
            }
            println!(
                "{indent_str}ByteString (len={}) {}",
                data.len(),
                hex::encode(data)
            );
        }
        Object::String(s) => {
            println!("{indent_str}String: {:?}", s);
        }
        Object::Long(items) => todo!(),
        Object::Int(i) => {
            println!("{indent_str}Int: {i} ({i:#X})");
        }
        Object::Float(_) => todo!(),
        Object::Complex(_, _) => todo!(),
    }
}
