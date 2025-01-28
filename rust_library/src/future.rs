use crate::RUNTIME;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub enum CassFutureResult<T, V> {
    NotReady,
    Result(T),
    Error(V),
    Completed,
}

impl<T, V> CassFutureResult<T, V> {
    pub fn is_err(&self) -> bool {
        matches!(self, CassFutureResult::Error(_))
    }
}

pub struct CassFuture<T, V> {
    pub result: Mutex<CassFutureResult<T, V>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
}

impl<T, V> CassFuture<T, V> {
    pub fn new_from_future<F>(fut: F) -> Arc<CassFuture<T, V>>
    where
        F: Future + Send + 'static,
        F::Output: Into<CassFutureResult<T, V>> + Send + 'static,
        T: Send + 'static,
        V: Send + 'static,
    {
        let cass_fut = Arc::new(CassFuture {
            result: Mutex::new(CassFutureResult::NotReady),
            join_handle: Mutex::new(None),
        });

        let cass_fut_clone = cass_fut.clone();
        *cass_fut.join_handle.lock().unwrap() = Some(RUNTIME.spawn(async move {
            let result = fut.await.into();
            if result.is_err() {
                *cass_fut_clone.result.lock().unwrap() = CassFutureResult::Error(result);
                return;
            }
            *cass_fut_clone.result.lock().unwrap() = CassFutureResult::Result(result);
        }));

        cass_fut
    }

    pub fn is_ready(&self) -> bool {
        match *self.result.lock().unwrap() {
            CassFutureResult::NotReady => false,
            _ => true,
        }
    }
}

// Implement Into<CassFutureResult<T, V>> for the supported output types
impl<T, V> From<Result<T, V>> for CassFutureResult<T, V> {
    fn from(res: Result<T, V>) -> Self {
        match res {
            Ok(res) => CassFutureResult::Result(res),
            Err(err) => CassFutureResult::Error(err),
        }
    }
}

impl<T, V> From<()> for CassFutureResult<T, V> {
    fn from(_: ()) -> Self {
        CassFutureResult::Completed
    }
}

// #[allow(unused)]
// trait CheckSendSync: Send + Sync {}
// impl<T, V> CheckSendSync for CassFuture<T, V> {}

