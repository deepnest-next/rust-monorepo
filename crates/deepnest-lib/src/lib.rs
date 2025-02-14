#[cfg(feature = "traits")]
pub use ::deepnest_types::traits;
pub use ::deepnest_types::types;

pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
