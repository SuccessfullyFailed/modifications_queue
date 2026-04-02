use std::sync::{ Arc, Condvar, Mutex, MutexGuard };



struct ModificationsQueueInner<T> {
	data:Mutex<Vec<Box<dyn FnMut(&T) + Send + Sync + 'static>>>,
	cond:Condvar
}
impl<T> Default for ModificationsQueueInner<T> {
	fn default() -> Self {
		ModificationsQueueInner {
			data: Mutex::new(Vec::new()),
			cond: Condvar::new()
		}
	}
}



pub struct ModificationsQueue<T>(Arc<ModificationsQueueInner<T>>);
impl<T> ModificationsQueue<T> {

	/// Create a new queue.
	pub fn new() -> ModificationsQueue<T> {
		Self::default()
	}

	/// Creates a remote to this queue.
	/// Allows changes from multiple places without having to borrow the queue.
	pub fn create_remote(&self) -> ModificationsQueueRemote<T> {
		ModificationsQueueRemote(Arc::clone(&self.0))
	}

	/// Add an item to the queue.
	pub fn add<Modification:FnMut(&T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.data.lock().unwrap().push(Box::new(modification));
	}

	/// Drain all modifications.
	pub fn drain(&self) -> Vec<Box<dyn FnMut(&T) + Send + Sync + 'static>> {
		self.0.data.lock().unwrap().drain(..).collect()
	}

	/// Apply all modifications to the given struct.
	pub fn apply_to(&self, target:&mut T) {
		for mut modification in self.drain() {
			modification(target);
		}
	}

	/// Puts the thread to sleep until anything is added to the queue.
	/// If something is already in the queue, it will immediately return that.
	/// Drains and returns all data in the queue.
	pub fn await_change(&self) -> Vec<Box<dyn FnMut(&T) + Send + Sync + 'static>> {
		let mut data_handle:MutexGuard<'_, Vec<Box<dyn FnMut(&T) + Send + Sync + 'static>>> = self.0.data.lock().unwrap();
		while data_handle.is_empty() {
			data_handle = self.0.cond.wait(data_handle).unwrap();
		}
		data_handle.drain(..).collect()
	}
}
impl<T> Default for ModificationsQueue<T> {
	fn default() -> Self {
		ModificationsQueue(Arc::new(ModificationsQueueInner::default()))
	}
}



pub struct ModificationsQueueRemote<T>(Arc<ModificationsQueueInner<T>>);
impl<T> ModificationsQueueRemote<T> {

	/// Add an item to the queue this remote targets.
	pub fn add<Modification:FnMut(&T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.data.lock().unwrap().push(Box::new(modification));
	}
}
impl<T> Clone for ModificationsQueueRemote<T> {
	fn clone(&self) -> Self {
		ModificationsQueueRemote(Arc::clone(&self.0))
	}
}