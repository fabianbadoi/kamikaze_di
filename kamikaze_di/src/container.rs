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
pub type Factory<T> = FnMut(&Container) -> T;
pub type Builder<T> = FnOnce(&Container) -> T;

enum DependencyType {
    Factory(Box<dyn Any>),
    Builder(Box<dyn Any>),
    Shared(Rc<Any>),
}

impl Container {
    pub fn register<T: 'static>(&mut self, item: T) -> DiResult<()> {
        let item = DependencyType::Shared(Rc::new(item));

        self.insert::<T>(item)
    }

    pub fn register_factory<T, F>(&mut self, factory: F) -> DiResult<()>
        where F: (FnMut(&Container) -> T) + 'static,
              T: 'static
    {
        let item = DependencyType::Factory(Box::new(factory));

        self.insert::<T>(item)
    }

    pub fn register_automatic_factory<T: 'static>(&mut self) -> DiResult<()> {
        let item = DependencyType::Factory(Box::new(|container| auto_factory::<T>(container)));

        self.insert::<T>(item)
    }

    pub fn register_builder<T, B>(&mut self, builder: B) -> DiResult<()>
        where B: (FnOnce(&Container) -> T) + 'static,
              T: 'static
    {
        let item = DependencyType::Builder(Box::new(builder));

        self.insert::<T>(item)
    }

    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        self.items.borrow().contains_key(&type_id)
    }

    pub fn get<T: 'static>(&self) -> ResolveResult<T> {
        let type_id = TypeId::of::<T>();
        let items = self.items.borrow();

        let item = items.get(&type_id);
        if item.is_none() {
            return Err(format!("Type not registered: {:?}", type_id));
        }

        use DependencyType::*;

        let item = match item.unwrap() {
            Factory(_) => self.call_factory::<T>(&type_id),
            Builder(_) => {
                self.consume_builder::<T>()?;
                self.get_shared(&type_id)
            },
            Shared(_) => self.get_shared(&type_id),
        };

        let raw = Rc::into_raw(item?);

        // this should be safe, considering proper registration
        return Ok(unsafe {
            Rc::<T>::from_raw(raw as *const T)
        });
    }

    pub fn call_factory<T: 'static>(&self, type_id: &TypeId) -> IntermediateResult {
        // we need to downcast the pointer before we use it, but to do that, we have to own it
        // so we have to remove it from the hash set and then put it back again
        if let DependencyType::Factory(factory) = self.items.borrow_mut().remove(&type_id).unwrap() {
            let mut factory = factory.downcast::<Box<Factory<T>>>().unwrap();
            let item = factory(self);

            let dependency = DependencyType::Factory(Box::new(factory));
            self.insert::<T>(dependency)?;

            return Ok(Rc::new(item));
        }

        panic!("Type {:?} not registered as factory", type_id)
    }

    fn consume_builder<T: 'static>(&self) -> DiResult<()> {
        let type_id = TypeId::of::<T>();

        if let DependencyType::Builder(builder) = self.items.borrow_mut().remove(&type_id).unwrap() {
            let builder = builder.downcast::<Box<Builder<T>>>().unwrap();
            let item = builder(self);
            let item = DependencyType::Shared(Rc::new(item));

            return self.insert::<T>(item);
        }

        panic!("Type {:?} not registered as builder", type_id)
    }

    fn get_shared(&self, type_id: &TypeId) -> IntermediateResult {
        if let DependencyType::Shared(item) = self.items.borrow().get(&type_id).unwrap() {
            return Ok(item.clone());
        }

        panic!("Type {:?} not registered as shared dependency", type_id)
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
type IntermediateResult = DiResult<Rc<dyn Any + 'static>>;
