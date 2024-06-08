use pyo3::prelude::*;

mod containers;
mod providers;

pub use containers::{DynamicContainer, WiringConfiguration};
pub use providers::{DependenciesContainer, Dependency, Provider};

#[pymodule]
fn inj(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Provider>()?;
    m.add_class::<Dependency>()?;
    m.add_class::<DependenciesContainer>()?;
    m.add_class::<WiringConfiguration>()?;
    m.add_class::<DynamicContainer>()?;
    Ok(())
}
