use std::{ sync::{ Arc, Mutex } };



pub struct ModificationsQueue<T>(Arc<Mutex<Vec<Box<dyn Fn(&mut T) + Send + Sync + 'static>>>>);
impl<T> ModificationsQueue<T> {

	/// Create a new queue.
	pub fn new() -> ModificationsQueue<T> {
		Self::default()
	}

	/// Creates a second modifications queue that is internally linked to this one.
	/// Allows changes from multiple places without having to borrow.
	pub fn create_remote(&self) -> ModificationsQueue<T> {
		ModificationsQueue(Arc::clone(&self.0))
	}

	/// Add an item to the queue.
	pub fn add<Modification:Fn(&mut T) + Send + Sync + 'static>(&self, modification:Modification) {
		self.0.lock().unwrap().push(Box::new(modification));
	}

	/// Drain all modifications.
	pub fn drain(&self) -> Vec<Box<dyn Fn(&mut T) + Send + Sync + 'static>> {
		self.0.lock().unwrap().drain(..).collect()
	}

	/// Apply all modifications to the given struct.
	pub fn apply_to(&self, target:&mut T) {
		let modifications:Vec<Box<dyn Fn(&mut T) + Send + Sync>> = self.drain();
		for modification in modifications {
			modification(target);
		}
	}
}
impl<T> Default for ModificationsQueue<T> {
	fn default() -> Self {
		ModificationsQueue(Arc::new(Mutex::new(Vec::new())))
	}
}



#[cfg(test)]
#[test]
fn full_test() {
	struct StructThatHasQueue {
		fake_data:usize,
		queue:ModificationsQueue<StructThatHasQueue>
	}
	let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
	assert_eq!(instance.fake_data, 0);
	
	// Test execute modification.
	instance.queue.add(|s| s.fake_data += 1);
	for modification in instance.queue.drain() {
		modification(&mut instance);
	}
	assert_eq!(instance.fake_data, 1);
	
	// Test don't execute modification twice.
	for modification in instance.queue.drain() {
		modification(&mut instance);
	}
	assert_eq!(instance.fake_data, 1);
	
	// Test don't execute all modifications twice.
	instance.queue.add(|s| s.fake_data += 1);
	instance.queue.add(|s| s.fake_data += 2);
	for modification in instance.queue.drain() {
		modification(&mut instance);
	}
	assert_eq!(instance.fake_data, 4);

	// Test remote.
	let remote:ModificationsQueue<StructThatHasQueue> = instance.queue.create_remote();
	remote.add(|s| s.fake_data += 10);
	for modification in instance.queue.drain() {
		modification(&mut instance);
	}
	assert_eq!(instance.fake_data, 14);
}