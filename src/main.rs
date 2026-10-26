fn main() {
    let ts = String::from("*Equal Test\r\n");
    let ts_byte = String::as_bytes(&ts);
    println!("{:?}", ts_byte);
}
