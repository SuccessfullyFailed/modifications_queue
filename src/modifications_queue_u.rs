#[cfg(test)]
mod tests {
	use std::{ thread::{ self, sleep }, time::{ Duration, Instant } };
	use crate::{ ModificationsQueue, ModificationsQueueRemote };



	struct StructThatHasQueue {
		fake_data:usize,
		queue:ModificationsQueue<StructThatHasQueue>
	}
	


	#[test]
	fn test_add_and_drain_modifications() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
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
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();
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
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();
		let remote_wrapper:Box<dyn Fn()> = Box::new(move || remote.add(|s| s.fake_data += 2));
		remote_wrapper();
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 2);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_modifications_capture_mutable_variable() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();

		let mut instance_index:usize = 0;
		instance.queue.add(move |s| {
			instance_index += 1;
			s.fake_data += instance_index;
		});

		let mut remote_index:usize = 0;
		remote.add(move |s| {
			remote_index += 1;
			s.fake_data += remote_index;
		});

		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 2);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_multi_level_variable_capturing() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let test_fn:Box<dyn Fn(&mut StructThatHasQueue) + Send + Sync + 'static> = Box::new(|s| s.fake_data += 2);
		instance.queue.add(move |s| s.queue.add(test_fn));
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 0);
		for modification in instance.queue.drain() {
			modification(&mut instance);
		}
		assert_eq!(instance.fake_data, 2);
		assert!(instance.queue.drain().is_empty());
	}

	#[test]
	fn test_await_modification() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();

		thread::spawn(move || {
			let thread_birth:Instant = Instant::now();
			let modifications:Vec<Box<dyn FnOnce(&mut StructThatHasQueue) + Send + Sync>> = instance.queue.await_change();
			assert_eq!(instance.fake_data, 0);
			for modification in modifications {
				modification(&mut instance);
			}
			assert_eq!(instance.fake_data, 6);
			assert!(thread_birth.elapsed().as_millis() < 20, "Modification was not detected upon adding it.");
		});
		
		sleep(Duration::from_millis(10));
		remote.add(|s| s.fake_data += 2);
		remote.add(|s| s.fake_data *= 3);
		sleep(Duration::from_millis(100));
	}

	#[test]
	fn test_await_modification_timeout() {
		let mut instance:StructThatHasQueue = StructThatHasQueue { fake_data: 0, queue: ModificationsQueue::new() };
		let remote:ModificationsQueueRemote<StructThatHasQueue> = instance.queue.create_remote();

		thread::spawn(move || {

			// Assert break after timeout when no changes made.
			let modifications:Vec<Box<dyn FnOnce(&mut StructThatHasQueue) + Send + Sync>> = instance.queue.await_change_timeout(Duration::from_millis(10));
			let start:Instant = Instant::now();
			assert_eq!(instance.fake_data, 0);
			for modification in modifications {
				modification(&mut instance);
			}
			assert_eq!(instance.fake_data, 0);
			assert!(start.elapsed().as_millis() < 20);
			
			// Assert break when changes made before timeout.
			let start:Instant = Instant::now();
			let modifications:Vec<Box<dyn FnOnce(&mut StructThatHasQueue) + Send + Sync>> = instance.queue.await_change_timeout(Duration::from_millis(200));
			assert_eq!(instance.fake_data, 0);
			for modification in modifications {
				modification(&mut instance);
			}
			assert_eq!(instance.fake_data, 0);
			assert!(start.elapsed().as_millis() < 100);
		});
		
		sleep(Duration::from_millis(100));
		remote.add(|s| s.fake_data += 2);
		remote.add(|s| s.fake_data *= 3);
	}
}