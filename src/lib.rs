use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::{any::Any, sync::Arc};
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

// type OffListener<Args> = Arc<dyn FnMut(Arc<Mutex<dyn Callback<Args>>>) -> bool>;
type Listener = Arc<dyn Fn() + Send + Sync>;

pub struct Signal {
    sender: Sender<()>,
    listeners: Arc<RwLock<Vec<Arc<dyn Fn() + Send + Sync>>>>,
}

impl Signal {
    pub fn new() -> Self {
        let (sender, _receiver) = channel();
        let listeners: Arc<RwLock<Vec<Listener>>> = Arc::new(RwLock::new(Vec::new()));
        Signal { sender, listeners }
    }

    pub fn emit(&self) {
        // 发送一个空的消息给所有监听器
        let _ = self.sender.send(());

        // 调用所有监听器的函数
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener();
        }
    }

    pub fn listener<F>(&self, listener: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let listener = Arc::new(listener);
        let mut listeners = self.listeners.write().unwrap();
        listeners.push(listener);
    }

    pub fn off<F>(&self, listener: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        // 移除一个监听器
        let mut listeners = self.listeners.write().unwrap();
        listeners.retain(|l| {
            let ptr = l.as_ref() as *const dyn Fn() as *const ();
            let listener_ptr = &listener as *const dyn Fn() as *const ();
            ptr != listener_ptr
        });
    }

    pub fn close(&self) {
        // 关闭信号
        let _ = self.sender.send(()); // 发送一个空的消息给所有监听器
        let mut listeners = self.listeners.write().unwrap();
        listeners.clear();
    }
}

pub type SimpleSignal = Signal;

#[cfg(test)]
mod tests {

    use std::sync::Mutex;

    use super::*;
    #[test]
    fn test_multi_emit() {
        let signal = Signal::new();
        let count = Arc::new(RwLock::new(0));

        // 添加监听器
        let count_clone = count.clone();
        signal.listener(move || {
            let mut count = count_clone.write().unwrap();
            *count += 1;
        });

        // 发送通知并等待一段时间
        signal.emit();
        signal.emit();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 检查计数器是否被增加
        let count = count.read().unwrap();
        assert_eq!(*count, 2);
    }

    #[test]
    fn test_multi_listener() {
        let signal = Signal::new();
        let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

        // 添加两个监听器
        let count_clone = count.clone();
        signal.listener(move || {
            let mut count: std::sync::MutexGuard<i32> = count_clone.lock().unwrap();
            *count += 1;
        });

        let count_clone = count.clone();
        signal.listener(move || {
            let mut count = count_clone.lock().unwrap();
            *count += 1;
        });

        // 发送通知并等待一段时间
        signal.emit();
        std::thread::sleep(std::time::Duration::from_millis(100));

        let count_clone = count.clone();

        // // 检查计数器是否被增加
        let count = count.lock().unwrap();
        assert_eq!(*count, 2);

        // 移除第一个监听器
        signal.off(move || {
            let mut count = count_clone.lock().unwrap();
            *count += 1;
        });
    }
}
