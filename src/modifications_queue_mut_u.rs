#[cfg(test)]
mod tests {
	use crate::{ ModificationsQueueMut, ModificationsQueueRemoteMut };



	struct StructThatHasQueue {
		fake_data:usize,
		queue:ModificationsQueueMut<StructThatHasQueue>
	}
	


	#[test]
	fn test_add_and_drain_modifications() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueueMut::new() };
		instance.queue.add(|s| s.fake_data += 2);
		instance.queue.add(|s| s.fake_data *= 3);
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 6);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_add_and_drain_modifications_from_remote() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueueMut::new() };
		let remote:ModificationsQueueRemoteMut<StructThatHasQueue> = instance.queue.create_remote();
		remote.add(|s| s.fake_data += 2);
		remote.add(|s| s.fake_data *= 3);
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 6);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_add_and_drain_modifications_from_remote_from_remote_function() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueueMut::new() };
		let remote:ModificationsQueueRemoteMut<StructThatHasQueue> = instance.queue.create_remote();
		let remote_wrapper:Box<dyn Fn()> = Box::new(move || remote.add(|s| s.fake_data += 2));
		remote_wrapper();
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 2);
		assert!(instance.queue.drain().is_empty());
	}
}