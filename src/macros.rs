#![warn(missing_docs)]

macro_rules! build_n_args {
	( $name:ident [$($add:tt)*]: $($var:ident: $typevar:ident),* ) => (
		impl< $($typevar,)* Result> $name<($($typevar,)*), Result> {
			/**
			 * call inner function, consumes the box
			 */
			pub fn call(mut self $(, $var: $typevar)*) -> Result {
				(*self.func)(($($var ,)*))
			}
		}

		impl< $($typevar,)* Result, F: 'static + FnOnce($($typevar),*) -> Result $($add)*> From<F> for $name<($($typevar,)*), Result> {
			fn from(func: F) -> Self {
				let mut func = Some(func);
				$name{
					func: Box::new(move |($($var ,)*)| -> Result {
						// the outer box gets consumed on a call,
						// so the unwrap() here must always succeed

						// the unwrap() needs to be done in a place
						// where the concrete "F" type is known (due to
						// size issues); it can't be done after type
						// elimination (that is why Box<FnOnce> doesn't
						// work yet)
						func.take().unwrap()($($var),*)
					})
				}
			}
		}
	)
}
