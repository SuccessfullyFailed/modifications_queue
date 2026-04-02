use std::{ sync::{ Arc, Mutex } };



type ModificationsQueueInnerMut<T> = Arc<Mutex<Vec<Box<dyn FnMut(&mut T) + Send + Sync + 'static>>>>;



pub struct ModificationsQueueMut<T>(ModificationsQueueInnerMut<T>);
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
	pub fn add<Modification:FnMut(&mut T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.lock().unwrap().push(Box::new(modification));
	}

	/// Drain all modifications.
	pub fn drain(&self) -> Vec<Box<dyn FnMut(&mut T) + Send + Sync + 'static>> {
		self.0.lock().unwrap().drain(..).collect()
	}

	/// Apply all modifications to the given struct.
	pub fn apply_to(&self, target:&mut T) {
		let modifications:Vec<Box<dyn FnMut(&mut T) + Send + Sync>> = self.drain();
		for mut modification in modifications {
			modification(target);
		}
	}
}
impl<T> Default for ModificationsQueueMut<T> {
	fn default() -> Self {
		ModificationsQueueMut(Arc::new(Mutex::new(Vec::new())))
	}
}



pub struct ModificationsQueueRemoteMut<T>(ModificationsQueueInnerMut<T>);
impl<T> ModificationsQueueRemoteMut<T> {

	/// Add an item to the queue this remote targets.
	pub fn add<Modification:FnMut(&mut T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.lock().unwrap().push(Box::new(modification));
	}
}
impl<T> Clone for ModificationsQueueRemoteMut<T> {
	fn clone(&self) -> Self {
		ModificationsQueueRemoteMut(Arc::clone(&self.0))
	}
}