use crate::containers;

use crate::providers;
use pyo3::types::{PyDict, PyString, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};

create_exception!(inj, SchemaError, PyException);

pub struct SchemaProcessorV1 {
    schema: Py<PyDict>,
    container: Py<containers::Container>,
}

impl SchemaProcessorV1 {
    fn new(py: Python, schema: Py<PyDict>) -> PyResult<Self> {
        Ok(Self {
            schema: schema.clone_ref(py),
            container: py
                .get_type_bound::<containers::DynamicContainer>()
                .call0()?
                .downcast::<containers::Container>()?
                .clone()
                .into(),
        })
    }

    pub fn process(&mut self, py: Python) -> PyResult<()> {
        let container_schema =
            self.schema
                .bind(py)
                .get_item("container")?
                .ok_or(PyValueError::new_err(
                    "shema have no 'container' key or it is empty",
                ))?;

        let container_schema = container_schema.downcast::<PyDict>()?;
        self.create_providers(py, container_schema, None)?;
        self.setup_injections(py, container_schema, None)?;
        Ok(())
    }

    fn get_providers(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.container.bind(py).getattr("providers")?.into())
    }

    fn create_providers(
        &mut self,
        py: Python,
        provider_schema: &Bound<'_, PyDict>,
        container: Option<Py<containers::Container>>,
    ) -> PyResult<()> {
        let dynamic_container_type = py.get_type_bound::<containers::Container>();
        let provider_container_type = py.get_type_bound::<providers::Container>();
        let container = container.unwrap_or(self.container.clone());
        let container = container.bind(py);

        for (provider_name, data) in provider_schema.iter() {
            let data = data.downcast::<PyDict>()?;
            let provider = if let Some(provider_cls) = data.get_item("provider")? {
                let provider_cls: &str = provider_cls.extract()?;
                let provider_type = _get_provider_cls(provider_cls)?;
                provider_type.call0(py)?
            } else {
                provider_container_type
                    .call1((dynamic_container_type.clone(),))?
                    .into()
            };
            let provider = provider.bind(py);
            container.call_method1("set_provider", (provider_name, provider.clone()))?;

            if providers::Container::is_type_of_bound(provider) {
                self.create_providers(
                    py,
                    data,
                    Some(
                        provider
                            .downcast::<providers::Container>()?
                            .getattr("container")?
                            .downcast::<containers::Container>()?
                            .clone()
                            .into(),
                    ),
                )?;
            }
        }
        Ok(())
    }

    fn setup_injections(
        &mut self,
        py: Python,
        provider_schema: &Bound<'_, PyDict>,
        container: Option<Py<containers::Container>>,
    ) -> PyResult<()> {
        let container = match container {
            Some(c) => c,
            None => self.container.clone(),
        };

        for (provider_name, data) in provider_schema.iter() {
            let provider = container.getattr(py, provider_name.downcast_into()?)?;
            let provider = provider.bind(py);
            let mut args = Vec::<PyObject>::new();
            let kwargs = PyDict::new_bound(py);

            if let Ok(ref provides) = data.get_item("provides") {
                let provides = self._resolve_provides(py, provides)?;
                provider.call_method1("set_provides", (provides,))?;
            }

            if let Ok(ref arg_injections) = data.get_item("args") {
                for arg in arg_injections.iter()? {
                    args.push(self._resolve_injection(py, &arg?)?);
                }
            }

            if !args.is_empty() {
                provider.call_method1("add_args", PyTuple::new_bound(py, args))?;
            }

            if let Ok(ref kwarg_injections) = data.get_item("kwargs") {
                for (name, arg) in kwarg_injections.downcast::<PyDict>()?.iter() {
                    let injection = self._resolve_injection(py, &arg)?;
                    kwargs.set_item(name, injection)?;
                }
            }

            if !kwargs.is_empty() {
                provider.call_method("add_kwargs", (), Some(&kwargs))?;
            }

            if providers::Container::is_type_of_bound(provider) {
                self.setup_injections(
                    py,
                    data.downcast()?,
                    Some(
                        provider
                            .downcast::<providers::Container>()?
                            .getattr("container")?
                            .downcast::<containers::Container>()?
                            .clone()
                            .into(),
                    ),
                )?;
            }
        }

        Ok(())
    }

    fn _resolve_provides(&self, py: Python, provides: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if _is_str_starts_with_container(provides)? {
            let provides: &str = provides.extract()?;
            self._resolve_provider(py, &provides[10..])
        } else {
            _import_string(provides.extract()?)
        }
    }

    fn _resolve_injection(&self, py: Python, arg: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        Ok(match arg {
            _ if _is_str_starts_with_container(&arg)? => {
                let arg: &str = arg.extract()?;
                self._resolve_provider(py, &arg[10..])?
            }
            _ if PyDict::is_type_of_bound(&arg) => {
                let mut provider_args = Vec::<PyObject>::new();
                let provider_type = _get_provider_cls(arg.get_item("provider")?.extract()?)?;
                if let Ok(ref provides) = arg.get_item("provides") {
                    let provides = self._resolve_provides(py, provides)?;
                    provider_args.push(provides);
                }
                if let Ok(ref args) = arg.get_item("args") {
                    for provider_arg in args.iter()? {
                        let provider_arg = provider_arg?;
                        if _is_str_starts_with_container(&provider_arg)? {
                            let provider_arg: &str = provider_arg.extract()?;
                            provider_args.push(self._resolve_provider(py, &provider_arg[10..])?)
                        }
                    }
                }
                provider_type.call1(py, PyTuple::new_bound(py, provider_args))?
            }
            _ => arg.clone().into(),
        })
    }

    fn _resolve_provider(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let segments: Vec<&str> = name.split(".").collect();
        let mut provider = self.container.getattr(py, *segments.get(0).unwrap())?;

        for segment in &segments[1..] {
            let (segment, have_parentheses) =
                if let (Some(start), Some(end)) = (segment.find("("), segment.find(")")) {
                    ((segment[start..end + 1]).to_string(), true)
                } else {
                    (segment.to_string(), false)
                };

            provider = provider.getattr(py, segment.as_str())?;

            if have_parentheses {
                provider = provider.call0(py)?;
            }
        }
        Ok(provider)
    }
}

fn _get_provider_cls(provider_cls_name: &str) -> PyResult<Py<PyType>> {
    match _fetch_provider_cls_from_std(provider_cls_name) {
        Some(provider_type) => Ok(provider_type),
        None => _import_provider_cls(provider_cls_name),
    }
}

fn _fetch_provider_cls_from_std(provider_cls_name: &str) -> Option<Py<PyType>> {
    Python::with_gil(|py| match provider_cls_name {
        "Provider" => Some(providers::Provider::type_object_bound(py).into()),
        // TODO: add other classes
        _ => None,
    })
}

fn _import_provider_cls(provider_cls_name: &str) -> PyResult<Py<PyType>> {
    let result = _import_string(provider_cls_name);
    Python::with_gil(|py| {
        let provider_type = py.get_type_bound::<providers::Provider>();
        match result {
            Ok(cls) => match cls.downcast_bound::<PyType>(py) {
                Ok(cls) if cls.is_subclass(&provider_type).unwrap_or(false) => {
                    Ok(cls.clone().into())
                }
                _ => Err(SchemaError::new_err(format!(
                    "Provider class {} is not a subclass of providers base class",
                    provider_cls_name
                ))),
            },
            Err(err) => Err({
                let schema_err = SchemaError::new_err(format!(
                    "Can not import provider '{}'",
                    provider_cls_name
                ));
                schema_err.set_cause(py, Some(err));
                schema_err
            }),
        }
    })
}

fn _import_string(string_name: &str) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        if let Some((module_name, member_name)) = string_name.rsplit_once(".") {
            let module = PyModule::import_bound(py, module_name)?;
            let member = module.getattr(member_name)?;
            Ok(member.into_py(py))
        } else if !string_name.is_empty() {
            let module = py.import_bound("builtins")?;
            let member = module.getattr(string_name)?;
            Ok(member.into_py(py))
        } else {
            Err(PyValueError::new_err("string should not be empty"))
        }
    })
}

fn _is_str_starts_with_container(obj: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(PyString::is_type_of_bound(&obj)
        && obj
            .call_method1("startswith", ("container.",))?
            .extract::<bool>()?)
}

/// Build provider schema
pub fn build_schema(schema: Py<PyDict>) -> PyResult<Py<PyDict>> {
    Python::with_gil(|py| -> PyResult<Py<PyDict>> {
        let mut schema_processor = SchemaProcessorV1::new(py, schema)?;
        schema_processor.process(py)?;
        Ok(schema_processor
            .get_providers(py)?
            .downcast_bound(py)?
            .clone()
            .into())
    })
}
