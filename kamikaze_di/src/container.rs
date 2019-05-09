#![feature(specialization)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Container {
    items: RefCell<HashMap<TypeId, DependencyType>>,
}

// TODO these can be trait aliases, once that feature becomes stable
// TODO maybe unboxed closures? look for support
pub type Factory = FnMut(&Container) -> Box<dyn Any + 'static>;
pub type Builder = FnOnce(&Container) -> Box<dyn Any + 'static>;

enum DependencyType {
    Factory(Box<Factory>),
    Builder(Box<Builder>),
    Shared(Rc<Any>),
}

impl Container {
    pub fn register<T: 'static>(&mut self, item: T) -> DiResult<()> {
        let item = DependencyType::Shared(Rc::new(item));

        self.insert::<T>(item)
    }

    pub fn register_factory<T: Any + 'static>(&mut self, factory: Box<Factory>) -> DiResult<()> {
        let item = DependencyType::Factory(factory);

        self.insert::<T>(item)
    }

    pub fn register_automatic_factory<T: 'static>(&mut self) -> DiResult<()> {
        let item = DependencyType::Factory(Box::new(|container| auto_factory::<T>(container)));

        self.insert::<T>(item)
    }

    pub fn register_builder<T: 'static>(&mut self, builder: Box<Builder>) -> DiResult<()> {
        let item = DependencyType::Builder(builder);

        self.insert::<T>(item)
    }

    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        self.items.borrow().contains_key(&type_id)
    }

    fn insert<T: 'static>(&self, item: DependencyType) -> DiResult<()> {
        let type_id = TypeId::of::<T>();

        if self.has::<T>() {
            return Err(format!("Container already has {:?}", type_id));
        }

        self.items.borrow_mut().insert(type_id, item);

        Ok(())
    }
}



fn auto_factory<T>(container: &Container) -> Box<T> {
    unimplemented!()
}


pub type DiResult<T> = Result<T, String>;
pub type ResolveResult<T> = DiResult<Rc<T>>;
