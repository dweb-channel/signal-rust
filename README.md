# signal-rust

Signal transmitter at will

```rust
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
```

> Arc<Mutex<T>>将 Mutex<T>包装在 Arc 中，用于多线程共享所有权的情况。Arc 提供了引用计数的功能，可以让多个线程共享 Mutex<T>的可变引用。在需要多个线程同时访问 T 时，Arc<Mutex<T>>是一个不错的选择。

> Mutex<Arc<T>>将 Arc<T>包装在 Mutex 中，用于多线程共享引用的情况。Mutex 提供了互斥锁的功能，可以让多个线程安全地访问 Arc<T>，但每次只能有一个线程访问 T。在需要多个线程共享 T 的只读引用时，Mutex<Arc<T>>是一个不错的选择。
