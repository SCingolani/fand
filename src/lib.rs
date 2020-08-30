pub mod inputs;
/// Operations over iterators. See the [Operation trait](trait.Operation) for details.
/// Unfortunately this crate relies on defining two different sets of structs.
/// The intention is that the "outer" structs, suffixed with "Operation",
/// containts the settings defining an operation, is serializable, and
/// implements the Operation trait so that they can be applied on top of each
/// other. The idea is that one might have a collection of operations and then
/// fold it calling apply. The second layer of structs define the internal
/// implementation of the operation itself in the Iterator implementation (i.e.
/// the actual processing is done in the next method of the Iterator
/// implementation).
pub mod operations;
pub mod outputs;
