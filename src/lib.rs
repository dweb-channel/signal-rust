use std::{
    any::{Any},
    collections::HashSet,
    sync::{Arc, Mutex}, hash::{Hash,Hasher},
};
use lazy_static::lazy_static;

pub trait Callback<Args> :Send + Sync {
    fn call(&mut self, args: Args) -> Box<dyn Any>;
    fn clone_box(&self) -> Box<dyn Callback<Args>>;
}
impl<Args, F:Send + Sync> Callback<Args> for F
where
    F: Send + Sync + 'static + Clone + FnMut(Args) -> Box<dyn Any>,
{
    fn call(&mut self, args: Args) ->Box<dyn Any> {
        self(args)
    }
    fn clone_box(&self) -> Box<dyn Callback<Args>> {
        Box::new(self.clone())
    }
}

impl<Args> Clone for Box<dyn Callback<Args>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

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

 type  SimpleCallback = dyn Callback<()>;
 type OffListener<Args> = Arc<dyn FnMut(Box<dyn Callback<Args>>) -> bool + Send + Sync>;

// lazy_static! {
//     static ref CALLBACK: OffListener<()> = {
//         Arc::new(|callback: Arc<dyn Callback<()>>| {
//             true
//         })
//     };
// }


enum SignalCtor {
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
    listener_set: Mutex<HashSet<Box<dyn Callback<Args>>>>,
}

impl<Args: Clone> Signal<Args> {
   pub fn listen(&mut self, cb:  Box<dyn Callback<Args>+ 'static>)  {
        self.listener_set.lock().unwrap().insert(cb.clone());
        // let listener = Arc::new(move |cb| {
        //     self.listener_set.lock().unwrap().remove(&cb)
        // });
        // listener
    }

    pub fn off(&mut self, cb:  Box<dyn Callback<Args>>) -> bool {
        self.listener_set.lock().unwrap().remove(&cb)
    }

    pub fn emit(&mut self, args: Args) {
        let cbs = self.listener_set.lock().unwrap().clone();
        for   mut cb in cbs {
                 match cb.call(args.clone()).downcast::<SignalCtor>() {
                    Ok(sig) => match *sig {
                        SignalCtor::OFF => {
                           self.off(cb);
                            break;
                        }
                        SignalCtor::BREAK => break,
                    },
                    Err(_) => {}
            }
        }
    }

    pub fn clear(&mut self) {
        self.listener_set.lock().unwrap().clear();
    }
}




pub type SimpleSignal = Signal<()>;


#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
}
