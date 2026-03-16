/// Trait ValueObject — immutable, equality dựa trên giá trị.
pub trait ValueObject: Clone + PartialEq + Eq + std::fmt::Debug {
    type ValidationError;

    fn validate(&self) -> Result<(), Self::ValidationError>;
}
