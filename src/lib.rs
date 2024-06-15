use pyo3::prelude::*;

mod containers;
mod providers;
mod schema;

#[pymodule]
fn inj(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<providers::Provider>()?;
    m.add_class::<providers::Dependency>()?;
    m.add_class::<providers::DependenciesContainer>()?;
    m.add_class::<containers::WiringConfiguration>()?;
    m.add_class::<containers::DynamicContainer>()?;
    Ok(())
}
