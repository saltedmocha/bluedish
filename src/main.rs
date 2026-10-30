fn main() {
    let test_str = String::from("$5\r\nTest-1\r\nTest-2\r\nTest-3\r\nTest-4\r\nTest-5\r\n");
    let mut split = test_str.trim_start_matches('$').split("\r\n");
    println!("{}", split.next().unwrap());
    println!("{:?}", split.collect::<Vec<&str>>());
}
