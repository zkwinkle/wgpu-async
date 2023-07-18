use crate::AsyncBuffer;
use crate::{wgpu_future::PollLoop, WgpuFuture};
use std::ops::Deref;
use std::sync::Arc;
use wgpu::Device;

/// A wrapper around a [`wgpu::Device`] which shadows some methods to allow for callback-and-poll
/// methods to be made async.
#[derive(Clone, Debug)]
pub struct AsyncDevice {
    device: Arc<Device>,
    poll_loop: Arc<PollLoop>,
}

impl AsyncDevice {
    pub(crate) fn new(device: Arc<Device>) -> Self {
        Self {
            poll_loop: Arc::new(PollLoop::new(device.clone())),
            device,
        }
    }

    /// Converts a callback-and-poll `wgpu` method pair into a future.
    ///
    /// The function given is called immediately, usually initiating work on the GPU immediately, however
    /// the device is only polled once the future is awaited.
    ///
    /// # Example
    ///
    /// The `Buffer::map_async` method is made async using this method:
    ///
    /// ```
    /// # let _ = stringify! {
    /// let future = device.do_async(|callback|
    ///     buffer_slice.map_async(mode, callback)
    /// );
    /// let result = future.await;
    /// # };
    /// ```
    pub fn do_async<F, R>(&self, f: F) -> WgpuFuture<R>
    where
        F: FnOnce(Box<dyn FnOnce(R) + Send>),
        R: Send + 'static,
    {
        let future = WgpuFuture::new(self.device.clone(), self.poll_loop.clone());
        f(future.callback());
        future
    }

    /// Creates an [`AsyncBuffer`].
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> AsyncBuffer {
        AsyncBuffer {
            device: self.clone(),
            buffer: self.device.create_buffer(desc),
        }
    }
}
impl Deref for AsyncDevice {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
impl<T> AsRef<T> for AsyncDevice
where
    T: ?Sized,
    <AsyncDevice as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}
