# signal-rust
Signal transmitter at will

```rust
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
```



>Arc<Mutex<T>>将Mutex<T>包装在Arc中，用于多线程共享所有权的情况。Arc提供了引用计数的功能，可以让多个线程共享Mutex<T>的可变引用。在需要多个线程同时访问T时，Arc<Mutex<T>>是一个不错的选择。

>Mutex<Arc<T>>将Arc<T>包装在Mutex中，用于多线程共享引用的情况。Mutex提供了互斥锁的功能，可以让多个线程安全地访问Arc<T>，但每次只能有一个线程访问T。在需要多个线程共享T的只读引用时，Mutex<Arc<T>>是一个不错的选择。