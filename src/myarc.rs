use std::alloc::Layout;
use std::sync::atomic::{Ordering, AtomicUsize};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

pub struct MyArc<T: ?Sized> {
    inner: NonNull<ArcInner<T>>,
    _phantom: PhantomData<T>,
}

pub struct MyArcWeak<T: ?Sized> {
    inner: NonNull<ArcInner<T>>,
    _phantom: PhantomData<T>,
}

struct ArcInner<T: ?Sized> {
    weak: AtomicUsize,
    strong: AtomicUsize,
    t: T,
}

unsafe impl<T> Send for MyArc<T> where T: Send + ?Sized {}
unsafe impl<T> Sync for MyArc<T> where T: Sync + ?Sized {}

unsafe impl<T> Send for MyArcWeak<T> where T: Send + ?Sized {}
unsafe impl<T> Sync for MyArcWeak<T> where T: Sync + ?Sized {}

impl<T> ArcInner<T> where T: ?Sized {
    fn increment_weak(&self) {
        self.weak.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_strong(&self) {
        self.strong.fetch_add(1, Ordering::Relaxed);
    }
    
    fn decrement_weak(&self) {
        self.weak.fetch_sub(1, Ordering::Relaxed);
    }

    fn decrement_strong(&self) {
        self.strong.fetch_sub(1, Ordering::Relaxed);
    }

    fn has_weak(&self) -> bool {
        self.weak.load(Ordering::Relaxed) != 0
    }

    fn has_strong(&self) -> bool {
        self.strong.load(Ordering::Relaxed) != 0
    }
}

impl<T> MyArcWeak<T>
where
    T: ?Sized,
{
    fn from_inner(inner: NonNull<ArcInner<T>>) -> Self {
        let self_ = Self {
            inner,
            _phantom: PhantomData,
        };

        self_.increment_weak();

        self_
    }

    fn increment_weak(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.increment_weak();
    }

    fn decrement_weak(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.decrement_weak();
    }

    fn has_strong(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.has_strong()
    }

    fn has_weak(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.has_weak()
    }
}

impl<T> MyArc<T>
where
    T: ?Sized,
{
    fn from_inner(inner: NonNull<ArcInner<T>>) -> Self {
        let self_ = Self {
            inner,
            _phantom: PhantomData,
        };

        self_.increment_strong();

        self_
    }

    pub fn downgrade(&self) -> MyArcWeak<T> {
        MyArcWeak::from_inner(self.inner)
    }

    fn increment_strong(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.increment_strong();
    }

    fn decrement_strong(&self) {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.decrement_strong();
    }

    fn has_strong(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.has_strong()
    }

    fn has_weak(&self) -> bool {
        let inner = unsafe { &mut *self.inner.as_ptr() };
        inner.has_weak()
    }
}

impl<T> MyArcWeak<T>
where
    T: ?Sized,
{
    pub fn upgrade(&self) -> Option<MyArc<T>> {
        if self.has_strong() {
            Some(MyArc::from_inner(self.inner))
        } else {
            None
        }
    }
}

impl<T> MyArc<T>
where
    T: Sized,
{
    pub fn new(t: T) -> Self {
        let inner = Box::leak(Box::new(ArcInner {
            strong: AtomicUsize::new(0),
            weak: AtomicUsize::new(0),
            t,
        }));

        let inner = NonNull::new(inner).unwrap();

        Self::from_inner(inner)
    }
}

impl<T> Clone for MyArc<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_inner(self.inner)
    }
}

impl<T> Clone for MyArcWeak<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_inner(self.inner)
    }
}

impl<T> Drop for MyArc<T>
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
            std::alloc::dealloc(inner as *mut ArcInner<T> as *mut _, Layout::for_value(inner));
        }
    }
}

impl<T> Drop for MyArcWeak<T>
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

impl<T> Deref for MyArc<T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let inner = unsafe { &*self.inner.as_ptr() };
        &inner.t
    }
}
