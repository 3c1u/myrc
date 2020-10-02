mod myrc;
mod myarc;

struct DropTest(String);

impl DropTest {
    pub fn new() -> Self {
        println!("DropTest::new()");
        Self("冷泉院桐香".into())
    }

    pub fn test(&self) {
        println!("DropTest::test(): {}", self.0);
    }
}

impl Drop for DropTest {
    fn drop(&mut self) {
        println!("DropTest::drop(): {}", self.0);
    }
}

fn main() {
    test_rc();
    test_arc();
}

fn test_rc() {
    use myrc::MyRc;
    
    let test = MyRc::new(DropTest::new());
    test.test();

    println!("clone test -> test_1");
    let test_1 = test.clone();
    test_1.test();

    println!("drop test");
    drop(test);

    println!("downgrade");
    let weak = MyRc::<_>::downgrade(&test_1);
    println!("drop weak");
    drop(weak);

    println!("downgrade");
    
    let weak = MyRc::<_>::downgrade(&test_1);

    println!("upgrade");
    let strong = weak.upgrade().expect("failed to upgrade");
    strong.test();

    println!("drop test_1");
    drop(test_1);
    println!("drop strong");
    drop(strong);

    println!("try upgrade");
    if weak.upgrade().is_some() {
        panic!("pointer exists");
    }
}

fn test_arc() {
    use myarc::MyArc;
    use std::time::Duration;
    
    let test = MyArc::new(DropTest::new());
    test.test();

    println!("clone test -> test_1");
    let test_1 = test.clone();
    test_1.test();

    println!("drop test");
    drop(test);

    println!("downgrade");
    let weak = MyArc::<_>::downgrade(&test_1);
    println!("drop weak");
    drop(weak);

    println!("downgrade");
    
    let weak = MyArc::<_>::downgrade(&test_1);

    println!("upgrade");
    let strong = weak.upgrade().expect("failed to upgrade");
    strong.test();

    println!("drop test_1");
    drop(test_1);
    
    let t1 = std::thread::spawn(move || {
        println!("drop strong");
        std::thread::sleep(Duration::from_secs(1));
        drop(strong);
    });
    let t2 = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        println!("try upgrade");
        if weak.upgrade().is_some() {
            panic!("pointer exists");
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();
}
