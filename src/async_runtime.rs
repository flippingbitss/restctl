// for native, we implement Send
#[cfg(not(target_arch = "wasm32"))]
pub trait SendNativeOnly: Send {}

// for wasm, we don't need Send due to having only a single thread
#[cfg(target_arch = "wasm32")]
pub trait SendNativeOnly {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> SendNativeOnly for T {}

#[cfg(target_arch = "wasm32")]
impl<T> SendNativeOnly for T {}

// #[derive(Clone)]
pub struct AsyncRuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    runtime: tokio::runtime::Handle,
}

// impl futures that can be invoked from UI thread which is sync,
// that's why they have no return types because we can't await, channels or Arc<Mutex> should be used for
// polling the status of the future from UI thread
impl AsyncRuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native(tokio_handle: tokio::runtime::Handle) -> Self {
        Self {
            runtime: tokio_handle,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_web() -> Self {
        Self {}
    }

    // native impl
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_future<F>(&self, future: F)
    where
        F: Future<Output = ()> + SendNativeOnly + 'static,
    {
        self.runtime.spawn(future);
    }

    // wasm32 impl
    #[cfg(target_arch = "wasm32")]
    pub fn spawn_future<F>(&self, future: F)
    where
        F: Future<Output = ()> + SendNativeOnly + 'static,
    {
        wasm_bindgen_futures::spawn_local(future);
    }
}
