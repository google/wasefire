#![no_std]
wasefire::applet!();

pub fn main() {
    debug!("hello world");
}

#[cfg(test)]
mod tests {
    use wasefire_stub as _;

    #[test]
    fn assert_true() {
        assert!(true);
    }
}
