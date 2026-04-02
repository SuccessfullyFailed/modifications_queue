#[cfg(test)]
mod tests {
	use crate::{ ModificationsQueue, ModificationsQueueRemote };
	use std::sync::Mutex;



	struct StructThatHasQueue {
		fake_data:Mutex<usize>,
		queue:ModificationsQueue<StructThatHasQueue>
	}
	


	#[test]
	fn test_add_and_drain_modifications() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: Mutex::new(0), queue: ModificationsQueue::new() };
		instance.queue.add(|s| *s.fake_data.lock().unwrap() += 2);
		instance.queue.add(|s| *s.fake_data.lock().unwrap() *= 3);
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(*instance.fake_data.lock().unwrap(), 6);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_add_and_drain_modifications_from_remote() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: Mutex::new(0), queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();
		remote.add(|s| *s.fake_data.lock().unwrap() += 2);
		remote.add(|s| *s.fake_data.lock().unwrap() *= 3);
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(*instance.fake_data.lock().unwrap(), 6);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_add_and_drain_modifications_from_remote_from_remote_function() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: Mutex::new(0), queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();
		let remote_wrapper:Box<dyn Fn()> = Box::new(move || remote.add(|s| *s.fake_data.lock().unwrap() += 2));
		remote_wrapper();
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(*instance.fake_data.lock().unwrap(), 2);
		assert!(instance.queue.drain().is_empty());
	}
}