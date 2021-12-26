//! Polymorphic script fn parameter types

#[allow(dead_code)] // Ignore unused if scripting is disabled
#[derive(Clone, Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
