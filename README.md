# json_stream_reader

No memory overhead lightning fast JSON Stream Reader.

```rust
let buf = "{\"foo1\": \"bar1\", \"foo2\": \"bar2\", \"foo3\": { \"foo4\": \"bar4\" }, \"foo5\": [ \"bar5\", \"bar6\" ] }".as_bytes();

let mut obj = vec![];
let r = RefCell::new(obj);
let mut reader = JsonStreamReader::new();
let result = reader.read(
    buf,
    0,
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
        r.borrow_mut().push(format!("key: {:}", obj_key.to_string()));
    },
    |obj_val: &str| {
        r.borrow_mut().push(format!("val: {:}", obj_val.to_string()));
    },
);

assert_eq!(result.is_ok(), true);
assert_eq!(*r.borrow(), vec![
    "{",
    "key: foo1",
    "val: bar1",

    "key: foo2",
    "val: bar2",

    "key: foo3",
    "{",

    "key: foo4",
    "val: bar4",

    "}",

    "key: foo5",

    "[",
    "val: bar5",
    "val: bar6",
    "]",

    "}"
]);
```