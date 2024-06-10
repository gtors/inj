use pyo3::prelude::*;
use pyo3::types::PyDict;

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};

create_exception!(inj, SchemaError, PyException);

#[pyclass]
pub struct SchemaProcessorV1 {
    _schema: PyObject,
    _container: PyObject,
}

#[pymethods]
impl SchemaProcessorV1 {
    #[new]
    fn new(schema: &PyDict) -> Self {
        let container = PyDict::new();
        Self {
            _schema: schema.into(),
            _container: container.into(),
        }
    }

    fn process(&mut self) {
        self._create_providers(self._schema.get_item("container").unwrap());
        self._setup_injections(self._schema.get_item("container").unwrap());
    }

    fn get_providers(&self) -> PyObject {
        self._container.get_item("providers").unwrap().clone()
    }

    fn _create_providers(&mut self, provider_schema: &PyDict, container: Option<&PyDict>) {
        let container = container.unwrap_or(&self._container);
        for (provider_name, data) in provider_schema.into_iter() {
            let provider = if let Some(provider_cls) = data.get_item("provider") {
                let provider_type = _get_provider_cls(provider_cls);
                let args = Vec::new();
                // ...
                provider_type.call(args, None).unwrap()
            } else {
                // ...
                PyDict::new().into()
            };
            container.set_item(provider_name, provider);
            if let Some(provider) = provider.downcast::<PyDict>() {
                self._create_providers(provider_schema, Some(provider));
            }
        }
    }

    fn _setup_injections(&mut self, provider_schema: &PyDict, container: Option<&PyDict>) {
        let container = container.unwrap_or(&self._container);
        for (provider_name, data) in provider_schema.into_iter() {
            let provider = container.get_item(provider_name).unwrap();
            let args = Vec::new();
            let kwargs = PyDict::new();
            // ...
            if let Some(provides) = data.get_item("provides") {
                // ...
                provider.setattr("provides", provides);
            }
            // ...
            if let Some(arg_injections) = data.get_item("args") {
                // ...
                for arg in arg_injections.into_iter() {
                    // ...
                    args.push(arg);
                }
            }
            // ...
            if let Some(kwarg_injections) = data.get_item("kwargs") {
                // ...
                for (name, arg) in kwarg_injections.into_iter() {
                    // ...
                    kwargs.set_item(name, arg);
                }
            }
            // ...
            if let Some(provider) = provider.downcast::<PyDict>() {
                self._setup_injections(provider_schema, Some(provider));
            }
        }
    }

    fn _resolve_provider(&self, name: &str) -> Option<PyObject> {
        // ...
        let mut segments = name.split(".");
        let mut provider = self._container.get_item(segments.next().unwrap())?;
        for segment in segments {
            // ...
            provider = provider.get_item(segment)?;
        }
        Some(provider)
    }
}

fn _get_provider_cls(provider_cls_name: &str) -> PyObject {
    // ...
    let provider_type = _fetch_provider_cls_from_std(provider_cls_name);
    if provider_type.is_none() {
        _import_provider_cls(provider_cls_name)
    } else {
        provider_type
    }
}

fn _fetch_provider_cls_from_std(provider_cls_name: &str) -> Option<PyObject> {
    // ...
    let provider_type = PyModule::import("providers")?.getattr(provider_cls_name)?;
    if provider_type.is_instance_of::<PyType>() {
        Some(provider_type)
    } else {
        None
    }
}

fn _import_provider_cls(provider_cls_name: &str) -> Option<PyObject> {
    // ...
    let module_name = provider_cls_name.split(".").collect::<Vec<_>>()[..-1].join(".");
    let module = PyModule::import(module_name)?;
    let cls = module.getattr(provider_cls_name)?;
    if cls.is_instance_of::<PyType>() {
        Some(cls)
    } else {
        None
    }
}

fn _import_string(string_name: &str) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let segments: Vec<&str> = string_name.split(".").collect();
        let (module_parts, member_name) = match &segments[..] {
            [member_name] => (&["builtins"][..], *member_name),
            [member_parts @ .., member_name] => (member_parts, *member_name),
            _ => return Err(PyValueError::new_err("string should not be empty")),
        };
        let module_name = module_parts.join(".");
        let module = PyModule::import_bound(py, module_name.as_str())?;
        let member = module.getattr(member_name)?;
        Ok(member.into_py(py))
    })
}

/// Build provider schema
#[pyfunction]
pub fn build_schema(schema: &PyDict) -> PyResult<PyDict> {
    let mut schema_processor = SchemaProcessorV1::new(schema);
    schema_processor.process();
    Ok(schema_processor.get_providers().into())
}
