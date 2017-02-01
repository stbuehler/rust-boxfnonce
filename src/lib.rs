/// `BoxFnOnce` boxes any `FnOnce` function up to a certain number of
/// arguments (10 as of now).
///
/// As `Box<FnOnce()>` doesn't work yet, and `Box<FnBox()>` will not be
/// available in stable rust, `BoxFnOnce` tries to provide a safe
/// implementation.
///
/// Instead of `Box<FnOnce(Args...) -> Result>` (or `Box<FnBox(Args...)
/// -> Result>`) the box type is `BoxFnOnce<(Args...,), Result>`  (the
/// arguments are always given as tuple type).  If the function doesn't
/// return a value (i.e. the empty tuple) `Result` can be ommitted:
/// `BoxFnOnce<(Args...,)>`.
///
/// Internally it constructs a FnMut which keeps the FnOnce in an
/// Option, and extracts the FnOnce on the first call.
///
/// You can build boxes for diverging functions too, but specifying the
/// type (like `BoxFnOnce<(), !>`) is not possible as the `!` type is
/// experimental.
///
/// # Examples
///
/// Move value into closure and box it:
///
/// ```
/// use boxfnonce::BoxFnOnce;
/// let s = String::from("foo");
/// let f : BoxFnOnce<()> = BoxFnOnce::from(|| {
///     println!("Got called: {}", s);
///     drop(s);
/// });
/// f.call();
/// ```
///
/// Move value into closure to return it, and box the closure:
///
/// ```
/// use boxfnonce::BoxFnOnce;
/// let s = String::from("foo");
/// let f : BoxFnOnce<(), String> = BoxFnOnce::from(|| {
///     println!("Got called: {}", s);
///     s
/// });
/// assert_eq!(f.call(), "foo".to_string());
/// ```
pub struct BoxFnOnce<Args, Result = ()> {
	/// theoretically we could call the inner FnMut multiple times,
	/// but the public accessors make sure this won't happen.
	func: Box<FnMut(Args) -> Result>,
}

impl<Args, Result> BoxFnOnce<Args, Result> {
	/// call inner function, consumes the box.
	///
	/// `call_tuple` can be used if the arguments are available as tuple.
	/// Each usable instance of BoxFnOnce<(...), Result> has a separate
	/// `call` method for passing arguments "untupled".
	pub fn call_tuple(mut self, args: Args) -> Result {
		(*self.func)(args)
	}

	/// `BoxFnOnce::new` is an alias for `BoxFnOnce::from`.
	pub fn new<F>(func: F) -> Self
		where Self: From<F>
	{
		Self::from(func)
	}
}

// implementation for zero arguments
impl<Result> BoxFnOnce<(), Result> {
	/**
	 * call inner function, consumes the box
	 */
	pub fn call(mut self) -> Result {
		(*self.func)(())
	}
}

impl<Result, F: 'static + FnOnce() -> Result> From<F> for BoxFnOnce<(), Result> {
	fn from(func: F) -> Self {
		let mut func = Some(func);
		BoxFnOnce{
			func: Box::new(move |_| -> Result {
				// the outer box gets consumed on a call,
				// so the unwrap() here must always succeed

				// the unwrap() needs to be done in a place
				// where the concrete "F" type is known (due to
				// size issues); it can't be done after type
				// elimination (that is why Box<FnOnce> doesn't
				// work yet)
				func.take().unwrap()()
			})
		}
	}
}

macro_rules! build_n_args {
	( $($var:ident: $typevar:ident),* ) => (
		impl< $($typevar),*, Result> BoxFnOnce<($($typevar),*,), Result> {
			/**
			 * call inner function, consumes the box
			 */
			pub fn call(mut self, $($var: $typevar),*) -> Result {
				(*self.func)(($($var),*,))
			}
		}

		impl< $($typevar),*, Result, F: 'static + FnOnce($($typevar),*) -> Result> From<F> for BoxFnOnce<($($typevar),*,), Result> {
			fn from(func: F) -> Self {
				let mut func = Some(func);
				BoxFnOnce{
					func: Box::new(move |($($var),*,)| -> Result {
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

build_n_args!(a1: A1);
build_n_args!(a1: A1, a2: A2);
build_n_args!(a1: A1, a2: A2, a3: A3);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8, a9: A9);
build_n_args!(a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8, a9: A9, a10: A10);

#[cfg(test)]
mod test {
	use super::BoxFnOnce;

	#[derive(PartialEq,Eq,Debug)]
	struct Arg1{}

	#[derive(PartialEq,Eq,Debug)]
	struct Arg2{}

	#[derive(PartialEq,Eq,Debug)]
	struct Arg3{}

	#[derive(PartialEq,Eq,Debug)]
	struct Arg4{}

	#[test]
	fn test_arg0() {
		let f = {
			let s = String::from("abc");
			move || -> String {
				(s)
			}
		};
		let f = BoxFnOnce::from(f);
		assert_eq!(f.call(), "abc".to_string());
	}

	#[test]
	fn test_arg1() {
		let f = {
			let s = String::from("abc");
			move |a| -> (String, Arg1) {
				(s, a)
			}
		};
		let f : BoxFnOnce<(Arg1,), (String, Arg1)> = BoxFnOnce::from(f);
		assert_eq!(f.call(Arg1{}), ("abc".into(), Arg1{}));
	}

	#[test]
	fn test_arg2() {
		let f = {
			let s = String::from("abc");
			move |a, b| -> (String, Arg1, Arg2) {
				(s, a, b)
			}
		};
		let f = BoxFnOnce::from(f);
		assert_eq!(f.call(Arg1{}, Arg2{}), ("abc".into(), Arg1{}, Arg2{}));
	}

	#[test]
	fn test_arg3() {
		let f = {
			let s = String::from("abc");
			move |a, b, c| -> (String, Arg1, Arg2, Arg3) {
				(s, a, b, c)
			}
		};
		let f = BoxFnOnce::from(f);
		assert_eq!(f.call(Arg1{}, Arg2{}, Arg3{}), ("abc".into(), Arg1{}, Arg2{}, Arg3{}));
	}

	#[test]
	fn test_arg4_void() {
		let f = {
			let s = String::from("abc");
			move |a, b, c, d| {
				drop(a);
				drop(b);
				drop(c);
				drop(d);
				drop(s);
			}
		};
		let f = BoxFnOnce::from(f);
		f.call(Arg1{}, Arg2{}, Arg3{}, Arg4{});
	}

	#[test]
	#[should_panic(expected = "inner diverging")]
	fn test_arg4_diverging() {
		let f = {
			let s = String::from("abc");
			move |a, b, c, d| -> ! {
				drop(a);
				drop(b);
				drop(c);
				drop(d);
				drop(s);
				panic!("inner diverging");
			}
		};
		let f = BoxFnOnce::from(f);
		f.call(Arg1{}, Arg2{}, Arg3{}, Arg4{});
	}
}
