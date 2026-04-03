use std::{ time::Duration, sync::{ Arc, Condvar, Mutex, MutexGuard } };




struct ModificationsQueueInnerMut<T> {
	data:Mutex<Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>>>,
	cond:Condvar
}
impl<T> Default for ModificationsQueueInnerMut<T> {
	fn default() -> Self {
		ModificationsQueueInnerMut {
			data: Mutex::new(Vec::new()),
			cond: Condvar::new()
		}
	}
}



pub struct ModificationsQueueMut<T>(Arc<ModificationsQueueInnerMut<T>>);
impl<T> ModificationsQueueMut<T> {

	/// Create a new queue.
	pub fn new() -> ModificationsQueueMut<T> {
		Self::default()
	}

	/// Creates a remote to this queue.
	/// Allows changes from multiple places without having to borrow the queue.
	pub fn create_remote(&self) -> ModificationsQueueRemoteMut<T> {
		ModificationsQueueRemoteMut(Arc::clone(&self.0))
	}

	/// Add an item to the queue.
	pub fn add<Modification:FnOnce(&mut T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.data.lock().unwrap().push(Box::new(modification));
	}

	/// Drain all modifications.
	pub fn drain(&self) -> Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>> {
		self.0.data.lock().unwrap().drain(..).collect()
	}

	/// Apply all modifications to the given struct.
	pub fn apply_to(&self, target:&mut T) {
		let modifications:Vec<Box<dyn FnOnce(&mut T) + Send + Sync>> = self.drain();
		for modification in modifications {
			modification(target);
		}
	}

	/// Puts the thread to sleep until anything is added to the queue.
	/// If something is already in the queue, it will immediately return that.
	/// Drains and returns all data in the queue.
	pub fn await_change(&self) -> Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>> {
		let mut data_handle:MutexGuard<'_, Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>>> = self.0.data.lock().unwrap();
		while data_handle.is_empty() {
			data_handle = self.0.cond.wait(data_handle).unwrap();
		}
		data_handle.drain(..).collect()
	}

	/// Puts the thread to sleep until anything is added to the queue or the given duration has surpassed.
	/// If something is already in the queue, it will immediately return that.
	/// Drains and returns all data in the queue.
	pub fn await_change_timeout(&self, timeout:Duration) -> Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>> {
		let mut data_handle:MutexGuard<'_, Vec<Box<dyn FnOnce(&mut T) + Send + Sync + 'static>>> = self.0.data.lock().unwrap();
		while data_handle.is_empty() {
			data_handle = self.0.cond.wait_timeout(data_handle, timeout).unwrap().0;
		}
		data_handle.drain(..).collect()
	}
}
impl<T> Default for ModificationsQueueMut<T> {
	fn default() -> Self {
		ModificationsQueueMut(Arc::new(ModificationsQueueInnerMut::default()))
	}
}



pub struct ModificationsQueueRemoteMut<T>(Arc<ModificationsQueueInnerMut<T>>);
impl<T> ModificationsQueueRemoteMut<T> {

	/// Add an item to the queue this remote targets.
	pub fn add<Modification:FnOnce(&mut T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.data.lock().unwrap().push(Box::new(modification));
	}
}
impl<T> Clone for ModificationsQueueRemoteMut<T> {
	fn clone(&self) -> Self {
		ModificationsQueueRemoteMut(Arc::clone(&self.0))
	}
}