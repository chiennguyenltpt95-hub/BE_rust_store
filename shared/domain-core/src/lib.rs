//! # Domain Core
//! Shared domain building blocks theo DDD.
//! Mọi service đều dùng crate này làm nền tảng.

pub mod aggregate;
pub mod domain_event;
pub mod entity;
pub mod error;
pub mod pagination;
pub mod repository;
pub mod value_object;

pub use aggregate::AggregateRoot;
pub use domain_event::{DomainEvent, DomainEventEnvelope};
pub use entity::Entity;
pub use error::DomainError;
pub use pagination::{Page, PageRequest};
pub use repository::{ReadRepository, Repository};
pub use value_object::ValueObject;
