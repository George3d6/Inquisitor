macro_rules! plugins {
    ( $( $x:ident ),* ) => {
        $(extern crate $x;)*

        pub fn init() -> Vec<Box<ReceptorPlugin>> {
            let mut v: Vec<Box<ReceptorPlugin>> = vec!();
            $(
                match $x::new() {
                    Ok(x) => {info!("{} successfully loaded", x.name()); v.push(Box::new(x)) }
                    Err(err) => {error!("{}", err)}
                }
            )*
            v
        }
    }
}
