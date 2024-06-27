use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PyTuple, PyType};
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};

#[pyclass(module = "inj", subclass)]
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

#[pyclass(extends=Provider, module="inj", subclass)]
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

#[pyclass(extends=Provider, module="inj", subclass)]
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

// Container provider provides an instance of declarative container.
#[pyclass(extends=Provider, module="inj", subclass)]
pub struct Container {
    container_cls: Option<Py<PyType>>,
}

#[pymethods]
impl Container {
    #[new]
    fn new() -> (Self, Provider) {
        // def __init__(self, container_cls=None, container=None, **overriding_providers):
        //     """Initialize provider."""
        //     self.__container_cls = container_cls
        //     self.__overriding_providers = overriding_providers
        //
        //     if container is None and container_cls:
        //         container = container_cls()
        //         container.assign_parent(self)
        //     self.__container = container
        //
        //     if self.__container and self.__overriding_providers:
        //         self.apply_overridings()
        //
        //     self.__parent = None
        //
        //     super(Container, self).__init__()

        let this = Container {
            container_cls: None,
        };
        let base = Provider::new();
        (this, base)
    }
}

#[pyfunction]
pub fn traverse(
    py: Python,
    providers: Vec<Py<Provider>>,
    types: Option<Vec<Py<PyType>>>,
) -> PyResult<Bound<'_, PyIterator>> {
    let traverse = Traverse::new(py, providers, types);
    PyIterator::from_bound_object(traverse.into_py(py).bind(py))
}

// #[pyclass]
// struct TraversePyIter {
//     inner: IntoIterator<Item = Py<Provider>>,
// }
//
// #[pymethods]
// impl TraversePyIter {
//     fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
//         slf
//     }
//
//     fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<Provider>> {
//         slf.inner.next()
//     }
// }
//

#[pyclass]
pub struct Traverse {
    providers: Vec<Py<Provider>>,
    visited: HashSet<usize>,
    to_visit: VecDeque<Py<Provider>>,
    types: Option<Py<PyTuple>>,
}

impl Traverse {
    pub fn new(py: Python, providers: Vec<Py<Provider>>, types: Option<Vec<Py<PyType>>>) -> Self {
        Self {
            providers: providers.clone(),
            visited: HashSet::new(),
            to_visit: providers.clone().into(),
            types: types.map(|ty| PyTuple::new_bound(py, ty).unbind()),
        }
    }
    fn visit(&mut self, py: Python, visiting: Py<Provider>) -> PyResult<Py<Provider>> {
        self.visited.insert(visiting.as_ptr() as usize);
        visiting
            .bind(py)
            .getattr("related")?
            .extract::<Vec<Bound<'_, Provider>>>()?
            .iter()
            .filter(|x| self.visited.contains(&(x.as_ptr() as usize)))
            .for_each(|x| self.to_visit.push_back(x.clone().unbind()));
        return Ok(visiting);
    }
}

#[pymethods]
impl Traverse {
    fn __iter__(_self: PyRef<Self>) -> PyRef<Self> {
        _self
    }

    fn __next__(mut _self: PyRefMut<Self>, py: Python) -> Option<PyResult<Py<Provider>>> {
        while let Some(visiting) = _self.to_visit.pop_front() {
            match _self.visit(py, visiting) {
                Ok(provider) => {
                    if let Some(types) = &_self.types {
                        match provider.bind(py).is_instance(types.bind(py)) {
                            Ok(false) => continue,
                            Ok(true) => return Some(Ok(provider.clone())),
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        return Some(Ok(provider.clone()));
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}
