#[macro_export]
macro_rules! ok_or_continue {
	( $e:expr ) => (
		match $e {
			Ok(value) => value,
			Err(_e) => {
				continue;
			},
		}
	)
}

#[macro_export]
macro_rules! some_or_continue {
	( $e:expr ) => (
		match $e {
			Some(value) => value,
			None => {
				continue
			},
		}
	)
}