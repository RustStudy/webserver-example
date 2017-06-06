use std::io::prelude::*;
// TcpListener用来监听TCP链接
use std::net::TcpListener;
use std::net::TcpStream;

use std::fs::File;

fn main() {
    // bind方法返回一个TcpListener实例
    // 该bind方法会返回一个Result<T, E>
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    // incoming方法返回包含了流序列的迭代器
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // 声明缓存区来保存读入的数据
    let mut buffer = [0; 512];

    // 从stream里读取的字节放到buffer里
    stream.read(&mut buffer).unwrap();


    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // 如果是以GET /，则返回html
    // 否则返回404
    //  增加/sleep，用来验证请求很慢
    let (status_line, filename) = if buffer.starts_with(get) {
       ("HTTP/1.1 200 OK\r\n\r\n", "src/hello.html")
    } else if buffer.starts_with(sleep) {
       thread::sleep(Duration::from_secs(5));
       ("HTTP/1.1 200 OK\r\n\r\n", "src/hello.html")
    } else {
       ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "src/404.html")
    };

    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    // 写入response
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
