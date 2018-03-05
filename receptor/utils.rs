use std::collections::HashMap;
use std::vec::Vec;

use hyper::server::{Request};


pub fn get_url_params(req: &Request) -> HashMap<&str, &str> {
    let mut params = HashMap::new();
    match req.query() {
        Some(query_str_raw) => {
            let params_vec = query_str_raw.split('&');
            for param in params_vec {
                let param_pair: Vec<&str> = param.split('=').collect();
                params.insert(param_pair[0], param_pair[1]);
            }
        }
        None => {}
    }
    return params
}
