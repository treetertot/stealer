#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::general::*;
        let mut nums = Vec::with_capacity(10000);
        for i in 0..10000 {
            nums.push(i);
        }
        let fizzy = run(nums.iter(), |n| {
            if n % 3 == 0 {
                if n % 5 == 0 {
                    String::from("FizzBuzz")                    
                } else {
                    String::from("Fizz")
                }
            } else if n % 5 == 0 {
                String::from("Buzz")
            } else {
                format!("{}", n)
            }
        });
        if let Some(val) = fizzy {
            for i in val {
                println!("{}", i);
            }
        } else {
            panic!("AAAAAAAAAA");
        }
    }
    #[test]
    fn fastbuzz() {
        use crate::ranges::*;
        let pool = RangeRunner::new(7);
        let res = pool.run(0, 100, |n| {
            if n % 3 == 0 {
                if n % 5 == 0 {
                    Ok("Fizzbuzz")
                } else {
                    Ok("Fizz")
                }
            } else if n % 5 == 0 {
                Ok("Buzz")
            } else {
                Err(n)
            }
        }).unwrap();
        for r in res {
            match r {
                Ok(message) => println!("{}", message),
                Err(n) => println!("{}", n),
            }
        }
    }
}

pub mod general;
mod vecbuilder;
pub mod ranges;
