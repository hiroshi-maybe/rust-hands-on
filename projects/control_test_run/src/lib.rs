pub fn prints_and_returns10(a: i32) -> i32 {
    println!("I got the value {}", a);
    10
}

pub fn add_two(a: i32) -> i32 {
    private_add_two(a)
}

fn private_add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    // $ cargo test -- --test-threads=1
    // $ cargo test -- --show-output

    #[test]
    fn will_pass() {
        assert_eq!(10, prints_and_returns10(1));
    }

    #[test]
    #[ignore]
    fn will_fail() {
        assert_eq!(5, prints_and_returns10(2));
    }

    // $ cargo test add_test2
    //    -> Run one test case
    // $ cargo test add
    //    -> Run three test cases

    #[test]
    fn add_test1() {
        assert_eq!(4, private_add_two(2));
    }

    #[test]
    fn add_test2() {
        assert_eq!(5, private_add_two(3));
    }

    #[test]
    fn add_test3() {
        assert_eq!(102, private_add_two(100));
    }

    // $ cargo test -- --ignored

    #[test]
    #[ignore]
    fn expensive_test() {
        panic!("foo");
    }
}

