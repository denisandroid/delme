use anyhow::bail;
use rdev::{Key, listen};
use std::ops::Deref;
use anyhow::anyhow;

struct KeyboardListener;

#[repr(transparent)]
#[derive(Default)]
struct KeyStateTable<const N: usize>([KeyStateEntry; N])
where
	[KeyStateEntry; N]: Default;

#[derive(Debug, PartialEq, Clone)]
struct KeyStateEntry {
	key: Key,
	state: ButtonState,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
enum ButtonState {
	Pressed,

	#[default]
	Released,
}

impl ButtonState {
	#[inline]
	pub const fn invert(&mut self) {
		*self = match self {
			ButtonState::Pressed => ButtonState::Released,
			ButtonState::Released => ButtonState::Pressed,
		}
	}

	#[inline]
	pub const fn is_pressed(&self) -> bool {
		matches!(self, ButtonState::Pressed)
	}

	#[inline]
	pub const fn is_released(&self) -> bool {
		matches!(self, ButtonState::Released)
	}
}

impl KeyStateEntry {
	#[inline]
	pub const fn from_press(key: Key) -> Self {
		Self {
			key,
			state: ButtonState::Pressed,
		}
	}

	#[inline]
	pub const fn from_release(key: Key) -> Self {
		Self {
			key,
			state: ButtonState::Released,
		}
	}

	#[inline]
	#[allow(dead_code)]
	pub const fn set_state(&mut self, state: ButtonState) {
		self.state = state;
	}

	#[inline]
	pub const fn invert_state(&mut self) {
		self.state.invert();
	}

	#[inline]
	pub const fn set_key(&mut self, key: Key) {
		self.key = key;
	}

	#[inline]
	pub const fn is_pressed(&self) -> bool {
		self.state.is_pressed()
	}

	#[inline]
	#[allow(dead_code)]
	pub const fn is_released(&self) -> bool {
		self.state.is_released()
	}
}

impl Deref for KeyStateEntry {
	type Target = Key;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.key
	}
}

impl Default for KeyStateEntry {
	#[inline]
	fn default() -> Self {
		Self {
			key: Key::Backspace,
			state: Default::default(),
		}
	}
}

impl<const N: usize> KeyStateTable<N>
where
	[KeyStateEntry; N]: Default,
{
	#[allow(dead_code)]
	#[inline]
	pub fn contains_entry(&self, entry: &KeyStateEntry) -> bool {
		self.0.contains(entry)
	}

	#[inline]
	fn find_entry_mut(&mut self, entry: &KeyStateEntry) -> Option<&mut KeyStateEntry> {
		self.0.iter_mut().find(|elem| **elem == *entry)
	}
}

impl KeyboardListener {
	pub fn listen<const N: usize>(
		init_key_table: impl FnOnce(&'_ mut [KeyStateEntry; N]) + Send + Sync + 'static,
		mut event_handler: impl FnMut(&[KeyStateEntry; N], Key, ButtonState) + Send + Sync + 'static,
	) -> anyhow::Result<()>
	where
		[KeyStateEntry; N]: Default,
	{
		let mut key_state_table = KeyStateTable::default();
		init_key_table(&mut key_state_table.0);

		listen(move |e| match e.event_type {
			rdev::EventType::KeyPress(key) => {
				let entry = KeyStateEntry::from_release(key);
				if let Some(key_entry) = key_state_table.find_entry_mut(&entry) {
					key_entry.invert_state();

					let (key, state) = (key_entry.key, key_entry.state);
					event_handler(&key_state_table.0, key, state);
				}
			}
			rdev::EventType::KeyRelease(key) => {
				let entry = KeyStateEntry::from_press(key);
				if let Some(key_entry) = key_state_table.find_entry_mut(&entry) {
					key_entry.invert_state();

					let (key, state) = (key_entry.key, key_entry.state);
					event_handler(&key_state_table.0, key, state);
				}
			}
			_ => {}
		}).map_err(|e| anyhow!("{:?}", e))?; // TODO REFACTORING ME

		Ok(())
	}
}

fn main() {
	KeyboardListener::listen::<3>(
		|key_table| {
			key_table[0].set_key(Key::ShiftRight);
			key_table[1].set_key(Key::ShiftLeft);
			key_table[2].set_key(Key::F8);
		},
		|state_array, _key, _state| {
			if (state_array[0].is_pressed() || state_array[1].is_pressed())
				&& state_array[2].is_pressed()
			{
				println!("SHIFT + F8");
			}
		},
	)
	.unwrap();
}
