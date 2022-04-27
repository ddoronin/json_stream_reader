// #[macro_use]
// extern crate lazy_static;
use json_stream_reader::json_stream_reader::*;
use json_stream_reader::json_value::JsonValue;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path;

fn main() {
    let dir = env::current_dir().unwrap();
    let large_file_path = dir.join(path::PathBuf::from("data/large-file.json"));
    let mut file = File::open(large_file_path).unwrap();
    let mut buf = [0; 512];

    let obj = vec![];
    let r = std::cell::RefCell::new(obj);
    let time = std::time::SystemTime::now();
    let mut reader = JsonStreamReader::new();
    while let Ok(size) = file.read(&mut buf) {
        if size > 0 {
            let res = reader.read(
                &buf[0..size],
                || {
                    r.borrow_mut().push("{".to_string());
                },
                || {
                    r.borrow_mut().push("}".to_string());
                },
                || {
                    r.borrow_mut().push("[".to_string());
                },
                || {
                    r.borrow_mut().push("]".to_string());
                },
                |obj_key: &str| {
                    r.borrow_mut()
                        .push(format!("key: {:}", obj_key.to_string()));
                },
                |obj_val: JsonValue| {
                    let s = match obj_val {
                        JsonValue::String(str) => format!("str: {:}", str),
                        JsonValue::Number(str) => format!("num: {:}", str.parse::<f32>().unwrap()),
                        JsonValue::Bool(b) => format!("bool: {:}", b),
                        JsonValue::Null => "null".to_string(),
                    };
                    r.borrow_mut().push(s);
                },
            );
            if res.is_err() {
                println!("{:?}", res.err());
                break;
            }
        } else {
            break;
        }
    }
    println!("took: {:?}", time.elapsed().unwrap());
    println!("Number of nodes: {:?}", r.borrow().len());
    let data = &*r.borrow();
    data.into_iter()
        .skip(data.len() - 100)
        .for_each(|s| println!("{}", s));
}
