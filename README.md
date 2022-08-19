## Constructor 
Auto generate `setter` and `getter` for struct.

You can add args
- setter
  - if false, then won't generate setter for this field
- getter 
  - if false, then won't generate getter for this field
- rename_getter 
  - getter method name, if not set, then default use ${field_name}
- rename_setter
  - setter method name, if not set, then default use set_${field_name}

```rust
mod server {
    use construct::Constructor;

    #[derive(Debug, Constructor)]
    pub struct Server {
        #[cons(setter = false, rename_getter = get_host)]
        host: String,
        #[cons(getter = false, rename_setter = set_server_port)]
        port: u16,
    }

    impl Server {
        pub fn new() -> Self {
            Self {
                host: String::from("127.0.0.1"),
                port: 8080,
            }
        }
    }
}

#[test]
fn test() {
    use crate::server::Server;
    let mut s = Server::new();
    println!("host: {}", s.get_host());

    // not implement because rename
    // println!("port: {}", s.port());
    s.set_server_port(10);
    println!("{:?}", s);
}
```