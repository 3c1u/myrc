use std::alloc::Layout;
use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

pub struct MyRc<T: ?Sized> {
    inner: NonNull<RcInner<T>>,
    _phantom: PhantomData<T>,
}

pub struct MyWeak<T: ?Sized> {
    inner: NonNull<RcInner<T>>,
    _phantom: PhantomData<T>,
}

struct RcInner<T: ?Sized> {
    weak: Cell<usize>,
    strong: Cell<usize>,
    t: T,
}

impl<T> MyWeak<T>
where
    T: ?Sized,
{
    fn from_inner(inner: NonNull<RcInner<T>>) -> Self {
        let self_ = Self {
            inner,
            _phantom: PhantomData,
        };

        self_.increment_weak();

        self_
    }

    fn increment_weak(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.weak.set(inner.weak.get() + 1);
    }

    fn decrement_weak(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.weak.set(inner.weak.get() - 1);
    }

    fn has_strong(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.strong.get() != 0
    }

    fn has_weak(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.weak.get() != 0
    }
}

impl<T> MyRc<T>
where
    T: ?Sized,
{
    fn from_inner(inner: NonNull<RcInner<T>>) -> Self {
        let self_ = Self {
            inner,
            _phantom: PhantomData,
        };

        self_.increment_strong();

        self_
    }

    fn increment_strong(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.strong.set(inner.strong.get() + 1);
    }

    fn decrement_strong(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.strong.set(inner.strong.get() - 1);
    }

    fn has_strong(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.strong.get() != 0
    }

    fn has_weak(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.weak.get() != 0
    }

    pub fn downgrade(&self) -> MyWeak<T> {
        MyWeak::from_inner(self.inner)
    }
}

impl<T> MyWeak<T>
where
    T: ?Sized,
{
    pub fn upgrade(&self) -> Option<MyRc<T>> {
        if self.has_strong() {
            Some(MyRc::from_inner(self.inner))
        } else {
            None
        }
    }
}

impl<T> MyRc<T>
where
    T: Sized,
{
    pub fn new(t: T) -> Self {
        let inner = Box::leak(Box::new(RcInner {
            strong: Cell::new(0),
            weak: Cell::new(0),
            t,
        }));

        let inner = NonNull::new(inner).unwrap();

        Self::from_inner(inner)
    }
}

impl<T> Clone for MyRc<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_inner(self.inner)
    }
}

impl<T> Clone for MyWeak<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_inner(self.inner)
    }
}

impl<T> Drop for MyRc<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.decrement_strong();

        if self.has_strong() {
            return;
        }

        let inner = unsafe { &mut *self.inner.as_ptr() };

        unsafe {
            std::ptr::drop_in_place(&mut inner.t);
        }

        if self.has_weak() {
            return;
        }

        unsafe {
            std::alloc::dealloc(inner as *mut RcInner<T> as *mut _, Layout::for_value(inner));
        }
    }
}

impl<T> Drop for MyWeak<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.decrement_weak();

        let inner = self.inner.as_ptr();

        if self.has_weak() || self.has_strong() {
            return;
        }

        unsafe {
            let layout = Layout::for_value(&*inner);
            std::alloc::dealloc(inner as *mut _, layout);
        }
    }
}

impl<T> Deref for MyRc<T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let inner = unsafe { &*self.inner.as_ptr() };
        &inner.t
    }
}

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
