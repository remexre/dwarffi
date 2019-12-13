pub struct DivModResult {
    pub d: i32,
    pub m: i32,
}

pub fn divmod_example(n: i32, d: i32) -> DivModResult {
    DivModResult { d: n / d, m: n % d }
}

pub fn void_example() {
    println!("no args, no ret, no prob")
}

pub fn declared_void_example(x: Option<u64>) -> () {
    println!("x = {:?}", x)
}

pub fn noret_example() -> ! {
    loop {
        println!("loop");
    }
}

pub fn float_add(x: f32, y: f64) -> f64 {
    x as f64 + y
}

pub fn when_is_now() -> String {
    format!("{:?}", std::time::Instant::now())
}
