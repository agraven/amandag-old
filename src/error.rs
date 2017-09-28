#[macro_export]
macro_rules! impl_error {
	[ $( ($l:ident, $f:ty) ),* ] => {
		$(
			impl From<$f> for Error {
				fn from(err: $f) -> Error {
					Error::$l(err)
				}
			}
		)*
	}
}
