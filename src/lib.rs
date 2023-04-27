use std::{
    any::Any,
    borrow::BorrowMut,
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, RwLock},
};
pub trait Callback<Args>: Send + Sync {
    fn call(&mut self, args: Args) -> Box<dyn Any>;
}
impl<Args, F: Send + Sync> Callback<Args> for F
where
    F: Clone + FnMut(Args) -> Box<dyn Any>,
{
    fn call(&mut self, args: Args) -> Box<dyn Any> {
        self(args)
    }
}

// pub trait WarpCallback<Args> : Send {
//     fn init(&mut self,args:Args) -> SharingCallback<Args> {
//         Arc::new(Mutex::new(args))
//     }
// } 

type SimpleCallback = dyn Callback<()>;
type OffListener<Args> = Arc<dyn FnMut(Arc<Mutex<dyn Callback<Args>>>) -> bool>;
type SharingCallback<Args> =  Arc<Mutex<dyn Callback<Args>>>;

#[derive(Clone)]
pub enum SignalCtor {
    /**
     * 返回该值，会解除监听
     */
    OFF,
    /**
     * 返回该值，会让接下来的其它监听函数不再触发
     */
    BREAK,
}

pub struct Signal<Args> {
    listener_set: RwLock<HashSet<Arc<Mutex<dyn Callback<Args>>>>>,
}

impl<Args: Clone + 'static> Signal<Args> {
    pub fn new() -> Self {
        return Self {
            listener_set: RwLock::new(HashSet::new()),
        };
    }

    pub fn listen(self, cb:SharingCallback<Args>) -> OffListener<Args> {
        self.listener_set.write().unwrap().insert(cb.clone());
        let listener = Arc::new(move |cb| self.off(cb));
        listener
    }

    pub fn off(&self, cb:SharingCallback<Args>) -> bool {
        self.listener_set.write().unwrap().remove(&cb)
    }

    pub fn emit(&mut self, args: Args) {
        let mut cbs = self.listener_set.write().unwrap();
        for cb in cbs.borrow_mut().iter() {
            match cb.lock().unwrap().call(args) {
                signal => {
                    let signal_ctor = match signal.downcast_ref::<SignalCtor>() {
                        Some(SignalCtor::OFF) => {
                            let cb_ref = cb.clone();
                            // self.off(Box::new(cb_ref));
                            break;
                        }
                        Some(SignalCtor::BREAK) => break,
                        _ => panic!("Invalid signal type!"),
                    };
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.listener_set.write().unwrap().clear();
    }
}

pub type SimpleSignal = Signal<()>;

impl<Args> PartialEq for dyn Callback<Args> {
    fn eq(&self, other: &Self) -> bool {
        // 比较 vtable 是否相等
        std::ptr::eq(
            // as_ref() 转换为 &dyn Callback<Args>
            self as *const dyn Callback<Args>,
            other as *const dyn Callback<Args>,
        )
    }
}

impl<Args> Eq for dyn Callback<Args> {}

impl<Args> Hash for dyn Callback<Args> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 使用 vtable 的地址作为 hash 值
        std::ptr::hash(self, state)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    #[derive(Clone)]
    struct MyArgs {
        arg1: i32,
        arg2: i32,
    }

    struct MyCallback {}

    impl Callback<MyArgs> for MyCallback {
        fn call(&mut self, args: MyArgs) -> Box<dyn Any> {
            println!("Received signal with args: ({}, {})", args.arg1, args.arg2);
            Box::new(SignalCtor::BREAK)
        }
    }

    #[test]
    fn test_signal() {
        let mut signal = Signal::new();

        // Add a listener
        let cb = Arc::new(Mutex::new(MyCallback {}));
        let listener = signal.listen(cb);
        assert_eq!(signal.listener_set.read().unwrap().len(), 1);

        // Emit a signal
        signal.emit(MyArgs { arg1: 1, arg2: 2 });
        thread::sleep(std::time::Duration::from_secs(1));

        // Remove the listener
        signal.off(cb);
        assert_eq!(signal.listener_set.read().unwrap().len(), 0);

        // Emit a signal (should not call any listeners)
        signal.emit(MyArgs { arg1: 3, arg2: 4 });
        thread::sleep(std::time::Duration::from_secs(1));

        // Add another listener
        let cb2 =  Arc::new(Mutex::new(MyCallback {}))
        let listener2 = signal.listen(cb2);
        assert_eq!(signal.listener_set.read().unwrap().len(), 1);

        // Clear all listeners
        signal.clear();
        assert_eq!(signal.listener_set.read().unwrap().len(), 0);
    }
}
