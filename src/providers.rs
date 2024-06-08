use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;

#[pyclass(subclass)]
#[derive(Clone)]
pub struct Provider {
    #[pyo3(get, set)]
    pub overridden: Vec<PyObject>,
    last_overriding: Option<PyObject>,
    overrides: Vec<PyObject>,
    // async_mode: AsyncMode,
}

impl std::default::Default for Provider {
    fn default() -> Self {
        Self {
            overridden: Vec::new(),
            last_overriding: None,
            overrides: Vec::new(),
            // async_mode: AsyncMode::Undefined,
        }
    }
}

#[pymethods]
impl Provider {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    // /// Helper function for creating its delegates.
    // ///
    // /// ```
    // /// let provider = Factory::new::<object>();
    // /// let delegate = provider.delegate();
    // ///
    // /// let delegated = delegate();
    // ///
    // /// assert!(provider.is(delegated));
    // /// ```
    // #[pyo3(name = "delegate")]
    // fn delegate<'p>(&self, py: Python<'p>) -> PyResult<&PyAny> {
    //     let delegate = PyDelegate::new(py, self)?;
    //
    //     Ok(delegate)
    // }
    // /// Function for overriding providers.
    // ///
    // /// ```
    // /// let provider1 = Factory::new::<SomeClass>();
    // /// let provider2 = Factory::new::<ChildSomeClass>();
    // ///
    // /// provider1.override(provider2);
    // ///
    // /// let some_instance = provider1.call();
    // /// assert!(some_instance.is::<ChildSomeClass>());
    // /// ```
    // #[pyo3(name = "override")]
    // fn override<'p>(&mut self, py: Python<'p>, provider: &PyAny) -> PyResult<()> {
    //     // Keep track of last overriding provider
    //     self.__last_overriding = Some(provider.clone_ref(py));
    //
    //     // Add provider to overriding providers list
    //     self.overridden.push(provider.clone_ref(py));
    //
    //     Ok(())
    // }
}

#[pyclass(extends=Provider, subclass)]
#[derive(Default)]
pub struct Dependency {
    instance_of: Option<Py<PyType>>,
    default: Option<Py<PyAny>>,
    parent: Option<Py<PyObject>>,
}

#[pymethods]
impl Dependency {
    #[new]
    #[pyo3(signature = (instance_of=None, default=None, **kwargs))]
    fn new(
        // py: Python,
        instance_of: Option<Py<PyType>>,
        default: Option<Py<PyAny>>,
        kwargs: Option<Py<PyDict>>,
    ) -> (Self, Provider) {
        let this = Self {
            instance_of: instance_of,
            default: default,
            parent: None,
            // parent: kwargs.map(|kw| {
            //     let kw_ref = kw.bind(py);
            //     kw_ref
            //         .clone()
            //         .get("__parent__")
            //         .unwrap_or(None)
            //         .to_object()
            //         .ok()
            // }),
        };
        let base = Provider::new();
        (this, base)
    }
}

#[pyclass(extends=Provider, subclass)]
pub struct DependenciesContainer {
    providers: HashMap<String, Py<PyObject>>,
    parent: Option<Py<PyObject>>,
}

#[pymethods]
impl DependenciesContainer {
    #[new]
    #[pyo3(signature = (**dependencies))]
    fn new(dependencies: Option<HashMap<String, Py<PyAny>>>) -> (Self, Provider) {
        let this = DependenciesContainer {
            providers: HashMap::new(),
            parent: None,
        };
        let base = Provider::new();
        (this, base)
    }
}
