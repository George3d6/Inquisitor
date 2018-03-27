macro_rules! plugins {
    ( $( $x:ident ),* ) => {
        $(extern crate $x;)*

        pub fn init() -> Vec<Box<AgentPlugin>> {
            let mut v: Vec<Box<AgentPlugin>> = vec!();
            $(v.push(Box::new($x::new()));)*
            v
        }
    }
}
