use crate::providers;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};
use pyo3::PyTypeInfo;
use std::collections::HashMap;

// use py_async_futures::futures::future::Future;
// use py_async_futures::futures::FutureExt;
// use py_async_futures::futures::StreamExt;
// use py_async_futures::tokio::stream;
// use py_async_futures::tokio::sync::mpsc;
use pyo3::{PyAny, PyResult, Python};

#[pyclass]
#[derive(Clone)]
pub struct WiringConfiguration {
    #[pyo3(get, set)]
    pub modules: Vec<String>,
    #[pyo3(get, set)]
    pub packages: Vec<String>,
    #[pyo3(get, set)]
    pub from_package: Option<String>,
    #[pyo3(get, set)]
    pub auto_wire: bool,
}

/// Container wiring configuration
#[pymethods]
impl WiringConfiguration {
    #[new]
    #[pyo3(signature = (modules=None, packages=None, from_package=None, auto_wire=true))]
    fn new(
        modules: Option<Vec<String>>,
        packages: Option<Vec<String>>,
        from_package: Option<String>,
        auto_wire: bool,
    ) -> Self {
        Self {
            modules: modules.unwrap_or_else(Vec::new),
            packages: packages.unwrap_or_else(Vec::new),
            from_package,
            auto_wire,
        }
    }
}

impl Default for WiringConfiguration {
    fn default() -> Self {
        Self {
            modules: Vec::new(),
            packages: Vec::new(),
            from_package: None,
            auto_wire: true,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DynamicContainer {
    #[pyo3(get)]
    pub provider_type: Py<PyType>,
    #[pyo3(get)]
    pub providers: HashMap<String, Py<PyAny>>,
    #[pyo3(get)]
    pub overridden: Vec<Py<PyAny>>,
    #[pyo3(get)]
    pub parent: Option<Py<PyAny>>,
    #[pyo3(get)]
    pub declarative_parent: Option<Py<PyAny>>,
    #[pyo3(get)]
    pub wiring_config: WiringConfiguration,
    #[pyo3(get)]
    pub wired_to_modules: Vec<String>,
    #[pyo3(get)]
    pub wired_to_packages: Vec<String>,
    // #[pyo3(get)]
    // pub __self__: Py<Self>,
}

#[pymethods]
impl DynamicContainer {
    #[new]
    fn new(py: Python) -> PyResult<Self> {
        Ok(Self {
            provider_type: providers::Provider::type_object_bound(py).unbind(),
            providers: HashMap::new(),
            overridden: Vec::new(),
            parent: None,
            declarative_parent: None,
            wiring_config: WiringConfiguration::default(),
            wired_to_modules: Vec::new(),
            wired_to_packages: Vec::new(),
            // __self__: Py::new(Self)?,
        })
    }

    // fn __deepcopy__(&self, memo: &mut HashMap<usize, Py<PyAny>>) -> PyResult<Py<PyAny>> {
    //     if let Some(copied) = memo.get(&self.id()) {
    //         return Ok(copied.clone());
    //     }
    //
    //     let copied = Self::new()?;
    //     memo.insert(self.id(), copied.clone());
    //
    //     // copied.__self__ = providers::deepcopy(&self.__self__, memo)?;
    //     // for name in copied.__self__.alt_names.iter() {
    //     //     copied.set_provider(name, copied.__self__.clone())?;
    //     // }
    //
    //     copied.provider_type = PyType::from_type::<providers::Provider>();
    //     copied.overridden = providers::deepcopy(&self.overridden, memo)?;
    //     copied.wiring_config = copy_module::deepcopy(&self.wiring_config, memo)?;
    //     copied.declarative_parent = self.declarative_parent.clone();
    //
    //     for (name, provider) in providers::deepcopy(&self.providers, memo)?.iter() {
    //         copied.set_provider(name, provider)?;
    //     }
    //
    //     copied.parent = providers::deepcopy(&self.parent, memo)?;
    //
    //     Ok(copied.into())
    // }

    fn __setattr__(&mut self, py: Python, name: String, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let is_provider = providers::Provider::is_type_of_bound(value);
        if is_provider && name != "parent" {
            self.check_provider_type(py, value)?;
            self.providers.insert(name.clone(), value.into_py(py));

            // if isinstance(value, providers.CHILD_PROVIDERS):
            //     value.assign_parent(self)
        }
        Ok(())
    }
    //
    // fn __delattr__(&mut self, name: &str) -> PyResult<()> {
    //     self.providers.remove(name);
    //     Ok(())
    // }
    //

    /// Return dependency providers dictionary.
    ///
    /// Dependency providers can be both of :py:class:`dependency_injector.providers.Dependency` and
    /// :py:class:`dependency_injector.providers.DependenciesContainer`.
    ///
    /// :rtype:
    ///     dict[str, :py:class:`dependency_injector.providers.Provider`]
    /// """
    #[getter]
    fn dependencies(&self, py: Python) -> PyResult<HashMap<String, PyObject>> {
        let dependency_types = PyTuple::new_bound(
            py,
            &[
                providers::Dependency::type_object_bound(py),
                providers::DependenciesContainer::type_object_bound(py),
            ],
        );
        let deps = self
            .providers
            .iter()
            .filter(|(_name, provider)| {
                let provider_ref = provider.bind(py);
                provider_ref.is_instance(&dependency_types).unwrap_or(false)
            })
            .map(|(name, provider)| (name.clone(), provider.clone()))
            .collect();
        Ok(deps)
    }

    // fn traverse(&self, types: Option<&PyTuple>) -> PyResult<PyObject> {
    //     let providers_list: Vec<PyObject> = self.providers.values().cloned().collect();
    //     let providers_tuple = PyTuple::new(py, providers_list);
    //     let providers_module = py.import("dependency_injector.providers")?;
    //     providers_module.call_method1("traverse", (providers_tuple, types))
    // }
    //
    // fn set_provider(&mut self, name: String, provider: PyObject) -> PyResult<()> {
    //     self.__setattr__(py, name, provider)
    // }
    //
    // fn override(&mut self, py: Python, overriding: PyObject) -> PyResult<()> {
    //     let self_ref: PyObject = self.into();
    //     if overriding == self_ref {
    //         Err(pyo3::exceptions::PyValueError::new_err("Container cannot override itself"))
    //     } else {
    //         self.overridden.push(overriding.clone_ref(py));
    //         Ok(())
    //     }
    // }
    //
    // fn reset_override(&mut self) {
    //     self.overridden.clear();
    // }
    //
    //
    // fn __setattr__(&mut self, name: &str, value: &PyAny) -> PyResult<()> {
    //     if value.is_instance::<providers::Provider>()
    //         && !value.is_instance::<providers::Self>()
    //         && name != "parent"
    //     {
    //         _check_provider_type(self, value)?;
    //
    //         self.providers.insert(name.to_string(), value.clone());
    //
    //         if value.is_instance::<providers::CHILD_PROVIDERS>() {
    //             value.as_ref::<providers::CHILD_PROVIDERS>()?.assign_parent(self)?;
    //         }
    //     }
    //
    //     super::__setattr__(name, value)
    // }
    //
    // fn __delattr__(&mut self, name: &str) -> PyResult<()> {
    //     if self.providers.contains_key(name) {
    //         self.providers.remove(name);
    //     }
    //     super::__delattr__(name)
    // }
    //
    // #[property]
    // fn dependencies(&self) -> HashMap<String, Py<PyAny>> {
    //     self.providers
    //         .iter()
    //         .filter(|(_, provider)| provider.is_instance::<providers::Dependency>() || provider.is_instance::<providers::DependenciesContainer>())
    //         .map(|(name, provider)| (name.clone(), provider.clone()))
    //         .collect()
    // }
    //
    // fn traverse(&self, types: Option<Vec<Py<PyAny>>>) -> impl Iterator<Item = Py<PyAny>> {
    //     providers::traverse(self.providers.values(), types).map(|p| p.into())
    // }
    //
    // fn set_providers(&mut self, providers: HashMap<String, Py<PyAny>>) -> PyResult<()> {
    //     for (name, provider) in providers.iter() {
    //         self.__setattr__(name, provider)?;
    //     }
    //     Ok(())
    // }
    //
    // fn set_provider(&mut self, name: &str, provider: &PyAny) -> PyResult<()> {
    //     self.__setattr__(name, provider)
    // }
    //
    // fn override(&mut self, overriding: &Py<PyAny>) -> PyResult<()> {
    //     if overriding == self.__self__ {
    //         return Err(Py::new_err(format!("Container {self} could not be overridden with itself")));
    //     }
    //
    //     self.overridden.push(overriding.clone());
    //
    //     for (name, provider) in overriding.as_ref::<DynamicContainer>()?.providers.iter() {
    //         self.providers
    //             .get_mut(name)
    //             .ok_or_else(|| Py::new_err(format!("No provider named {name} found in container {self}")))?.override(provider)?;
    //     }
    //
    //     Ok(())
    // }
    //
    // fn override_providers(
    //     &mut self,
    //     overriding_providers: HashMap<String, Py<PyAny>>,
    // ) -> PyResult<ProvidersOverridingContext> {
    //     let overridden_providers = Vec::new();
    //     for (name, overriding_provider) in overriding_providers.iter() {
    //         let container_provider = self.providers.get(name).ok_or_else(|| Py::new_err(format!("No provider named {name} found in container {self}")))?;
    //         container_provider.override(overriding_provider)?;
    //         overridden_providers.push(container_provider.clone());
    //     }
    //     Ok(ProvidersOverridingContext {
    //         container: self,
    //         overridden_providers,
    //     })
    // }
    //
    // fn reset_last_overriding(&mut self) -> PyResult<()> {
    //     if self.overridden.is_empty() {
    //         return Err(Py::new_err(format!("Container {self} is not overridden")));
    //     }
    //
    //     self.overridden.pop();
    //
    //     for provider in self.providers.values() {
    //         provider.reset_last_overriding()?;
    //     }
    //
    //     Ok(())
    // }
    //
    // fn reset_override(&mut self) -> PyResult<()> {
    //     self.overridden.clear();
    //
    //     for provider in self.providers.values() {
    //         provider.reset_override()?;
    //     }
    //
    //     Ok(())
    // }
    //
    // fn is_auto_wiring_enabled(&self) -> bool {
    //     self.wiring_config.auto_wire
    // }
    //
    // fn wire(
    //     &mut self,
    //     modules: Option<Vec<String>>,
    //     packages: Option<Vec<String>>,
    //     from_package: Option<String>,
    // ) -> PyResult<()> {
    //     let modules = modules.unwrap_or_else(|| self.wiring_config.modules.clone());
    //     let packages = packages.unwrap_or_else(|| self.wiring_config.packages.clone());
    //
    //     let modules = modules.iter().map(|m| _resolve_string_imports(m, &from_package)).flatten().collect::<Vec<String>>();
    //     let packages = packages.iter().map(|p| _resolve_string_imports(p, &from_package)).flatten().collect::<Vec<String>>();
    //
    //     if modules.is_empty() && packages.is_empty() {
    //         return Ok(());
    //     }
    //
    //     wire(self, &modules, &packages)?;
    //
    //     if !modules.is_empty() {
    //         self.wired_to_modules.extend(modules);
    //     }
    //     if !packages.is_empty() {
    //         self.wired_to_packages.extend(packages);
    //     }
    //
    //     Ok(())
    // }
    //
    // fn unwire(&mut self) -> PyResult<()> {
    //     unwire(&self.wired_to_modules, &self.wired_to_packages)?;
    //
    //     self.wired_to_modules.clear();
    //     self.wired_to_packages.clear();
    //
    //     Ok(())
    // }
    //
    // fn init_resources(&mut self) -> PyResult<()> {
    //     let futures = self
    //         .traverse(Some(vec![self.provider_type.clone()]))?
    //         .map(|provider| {
    //             let provider = provider.downcast::<providers::Resource>()?;
    //             provider.init()
    //         })
    //         .collect::<Vec<_>>();
    //     if !futures.is_empty() {
    //         Ok(asyncio::gather(futures))
    //     } else {
    //         Ok(())
    //     }
    // }
    //
    // fn shutdown_resources(&mut self) -> PyResult<()> {
    //     let independent_resources = |resources: &mut [Py<providers::Resource>], initialized: &mut [bool]| -> PyIter<Py<providers::Resource>> {
    //         for resource in resources.iter() {
    //             for other_resource in resources.iter() {
    //                 if !other_resource.initialized {
    //                     continue;
    //                 }
    //                 if resource.related.contains(&other_resource) {
    //                     break;
    //                 }
    //             }
    //             if !initialized.contains(&true) {
    //                 yield resource.clone();
    //             }
    //         }
    //     };
    //
    //     let async_ordered_shutdown = |resources: &mut [Py<providers::Resource>], initialized: &mut [bool]| -> PyResult<()> {
    //         while initialized.contains(&true) {
    //             let resources_to_shutdown = independent_resources(resources, initialized).collect::<Vec<_>>();
    //             if resources_to_shutdown.is_empty() {
    //                 return Err(PyErr::new::<RuntimeError, _>("Unable to resolve resources shutdown order"));
    //             }
    //             let futures = resources_to_shutdown.iter().map(|resource| resource.shutdown()).collect::<Vec<_>>();
    //             asyncio::gather(futures).map_err(|e| PyErr::from(e))?;
    //         }
    //         Ok(())
    //     };
    //
    //     let sync_ordered_shutdown = |resources: &mut [Py<providers::Resource>], initialized: &mut [bool]| -> PyResult<()> {
    //         while initialized.contains(&true) {
    //             let resources_to_shutdown = independent_resources(resources, initialized).collect::<Vec<_>>();
    //             if resources_to_shutdown.is_empty() {
    //                 return Err(PyErr::new::<RuntimeError, _>("Unable to resolve resources shutdown order"));
    //             }
    //             for resource in resources_to_shutdown {
    //                 resource.shutdown();
    //             }
    //         }
    //         Ok(())
    //     };
    //
    //     let mut resources = self
    //         .traverse(Some(vec![self.provider_type.clone()]))?
    //         .map(|provider| provider.downcast::<providers::Resource>().unwrap())
    //         .collect::<Vec<_>>();
    //     let mut initialized = vec![false; resources.len()];
    //
    //     if resources.iter().any(|resource| resource.is_async_mode_enabled()) {
    //         async_ordered_shutdown(&mut resources, &mut initialized)
    //     } else {
    //         sync_ordered_shutdown(&mut resources, &mut initialized)
    //     }
    // }
    //
    // pub fn load_config(&self) -> PyResult<()> {
    //     let providers = self.providers.clone();
    //
    //     for provider in providers
    //         .lock()
    //         .unwrap()
    //         .values()
    //         .filter(|provider| provider.hasattr("load"))
    //     {
    //         let provider = provider.clone();
    //
    //         Python::with_gil(|py| {
    //             let args = PyList::new(py, 0);
    //             let kwargs = PyDict::new(py, 0);
    //
    //             provider.call_method("load", args, Some(kwargs))?;
    //
    //             Ok(())
    //         })?;
    //     }
    //
    //     Ok(())
    // }
    //
    // fn apply_container_providers_overridings(&mut self) {
    //     for provider in self.traverse(None) {
    //         provider.call_method1("apply_overridings", ());
    //     }
    // }
    //
    // fn reset_singletons(&mut self) -> SingletonResetContext {
    //     for provider in self.traverse(None) {
    //         provider.call_method1("reset", ());
    //     }
    //     SingletonResetContext { container: self }
    // }
    //
    // fn check_dependencies(&mut self) -> PyResult<()> {
    //     let undefined = self
    //         .traverse(None)?
    //         .call_method1("filter", (py().get("dependency_injector.providers")?.get("Dependency")?))?;
    //     let undefined = undefined.call_method0("list")?;
    //     let undefined = undefined
    //         .iter()
    //         .filter_map(|dependency| {
    //             if dependency.is_instance_of(py().get("dependency_injector.providers")?.get("Dependency")?)? {
    //                 Some(dependency.get("parent_name")?.as_str()?)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect::<Vec<_>>();
    //     if undefined.is_empty() {
    //         return Ok(());
    //     }
    //     let container_name = if let Some(parent_name) = self.parent_name() {
    //         parent_name
    //     } else {
    //         self.get_type().name().to_owned()
    //     };
    //     Err(py().new_err(format!(
    //         "Container \"{}\" has undefined dependencies: {}",
    //         container_name,
    //         undefined.join(", ")
    //     )))
    // }
    //
    // fn from_schema(&mut self, schema: &PyDict) -> PyResult<()> {
    //     for (name, provider) in schema {
    //         self.set_provider(name.as_str()?, provider)?;
    //     }
    //     Ok(())
    // }
    //
    // fn from_yaml_schema(&mut self, filepath: &str, loader: Option<Py<PyAny>>) -> PyResult<()> {
    //     let yaml = py().import("yaml")?;
    //     let loader = loader.unwrap_or_else(|| yaml.get("SafeLoader")?);
    //     let file = py().open(filepath, "r")?;
    //     let schema = yaml.call_method1("load", (file, loader))?;
    //     self.from_schema(schema)
    // }
    //
    // fn from_json_schema(&mut self, filepath: &str) -> PyResult<()> {
    //     let file = py().open(filepath, "r")?;
    //     let schema = py().import("json")?.call_method1("load", (file,))?;
    //     self.from_schema(schema)
    // }
    //
    // fn resolve_provider_name(&self, provider: &PyAny) -> PyResult<String> {
    //     for (name, container_provider) in &self.providers {
    //         if container_provider.as_ptr() == provider.as_ptr() {
    //             return Ok(name.to_owned());
    //         }
    //     }
    //     Err(py().new_err(format!("Can not resolve name for provider \"{:?}\"", provider)))
    // }
    //
    // fn parent_name(&self) -> Option<String> {
    //     self.parent.as_ref().map(|parent| parent.parent_name())
    //         .or_else(|| self.declarative_parent.as_ref().map(|parent| parent.name().to_owned()))
    // }
    //
    // fn assign_parent(&mut self, parent: &Py<DynamicContainer>) {
    //     self.parent = Some(parent);
    // }
}

impl DynamicContainer {
    fn check_provider_type(&self, py: Python, provider: &Bound<'_, PyAny>) -> PyResult<()> {
        if !provider.is_instance(&self.provider_type.bind(py)).is_ok() {
            // let error_msg = format!("{} can contain only {} instances", self, self.provider_type);
            let error_msg = "wrong provider type";
            Err(PyRuntimeError::new_err(error_msg))
        } else {
            Ok(())
        }
    }
}