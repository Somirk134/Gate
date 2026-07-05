use crate::error::AppError;

pub type ComponentKey = &'static str;

pub trait AppContext: Send + Sync {
    fn services(&self) -> &dyn ServiceRegistry;

    fn components(&self) -> &dyn ComponentRegistry;
}

pub trait ServiceRegistry: Send + Sync {
    fn contains(&self, key: ComponentKey) -> bool;
}

pub trait ComponentRegistry: Send + Sync {
    fn contains(&self, key: ComponentKey) -> bool;
}

pub trait Provider<T>: Send + Sync {
    fn provide(&self, context: &dyn AppContext) -> Result<T, AppError>;
}

pub trait Factory<T>: Send + Sync {
    fn create(&self, context: &dyn AppContext) -> Result<T, AppError>;
}
