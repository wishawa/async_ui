pub trait Track {
	type Path;
	type Tracked<S>;
}
pub type Tracked<T: Track, S> = T::Tracked<S>;