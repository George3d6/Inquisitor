macro_rules! plugins {
    ( $( $x:ident ),* ) => {
        $(extern crate $x;)*

        pub fn init() -> Vec<Box<AgentPlugin>> {
            let mut v: Vec<Box<AgentPlugin>> = vec!();
            $(
                match $x::new() {
                    Ok(x) => {v.push(Box::new(x)) }
                    Err(x) => {println!("{}", x)}
                }
            )*
            v
        }
    }
}
