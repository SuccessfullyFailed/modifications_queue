use std::{ sync::{ Arc, Mutex } };



type ModificationsQueueInner<T> = Arc<Mutex<Vec<Box<dyn Fn(&T) + Send + Sync + 'static>>>>;



pub struct ModificationsQueue<T>(ModificationsQueueInner<T>);
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
	pub fn add<Modification:Fn(&T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.lock().unwrap().push(Box::new(modification));
	}

	/// Drain all modifications.
	pub fn drain(&self) -> Vec<Box<dyn Fn(&T) + Send + Sync + 'static>> {
		self.0.lock().unwrap().drain(..).collect()
	}

	/// Apply all modifications to the given struct.
	pub fn apply_to(&self, target:&mut T) {
		for modification in self.drain() {
			modification(target);
		}
	}
}
impl<T> Default for ModificationsQueue<T> {
	fn default() -> Self {
		ModificationsQueue(Arc::new(Mutex::new(Vec::new())))
	}
}



pub struct ModificationsQueueRemote<T>(ModificationsQueueInner<T>);
impl<T> ModificationsQueueRemote<T> {

	/// Add an item to the queue this remote targets.
	pub fn add<Modification:Fn(&T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.lock().unwrap().push(Box::new(modification));
	}
}
impl<T> Clone for ModificationsQueueRemote<T> {
	fn clone(&self) -> Self {
		ModificationsQueueRemote(Arc::clone(&self.0))
	}
}