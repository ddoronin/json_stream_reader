use json_stream_reader::json_stream_reader::*;
use json_stream_reader::json_token::JsonToken;
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
            let res = reader.read(&buf[0..size]);
            match res {
                Ok(tokens) => {
                    for token in tokens {
                        match token {
                            JsonToken::ObjBeg => r.borrow_mut().push("{".to_string()),
                            JsonToken::ObjEnd => r.borrow_mut().push("}".to_string()),
                            JsonToken::ArrBeg => r.borrow_mut().push("[".to_string()),
                            JsonToken::ArrEnd => r.borrow_mut().push("]".to_string()),
                            JsonToken::Key(obj_key) => r
                                .borrow_mut()
                                .push(format!("key: {:}", obj_key.to_string())),
                            JsonToken::Val(JsonValue::String(str)) => {
                                r.borrow_mut().push(format!("str: {:}", str))
                            }
                            JsonToken::Val(JsonValue::Number(str)) => r
                                .borrow_mut()
                                .push(format!("num: {:}", str.parse::<f32>().unwrap())),
                            JsonToken::Val(JsonValue::Bool(b)) => {
                                r.borrow_mut().push(format!("bool: {:}", b))
                            }
                            JsonToken::Val(JsonValue::Null) => {
                                r.borrow_mut().push("null".to_string())
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
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
