use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Default)]
pub struct CycleStopper {
    tracked: RefCell<HashSet<TypeId>>,
}

impl CycleStopper {
    pub fn track(&self, type_id: TypeId) -> CycleGuard<'_> {
        let mut tracked = self.tracked.borrow_mut();

        if tracked.contains(&type_id) {
            panic!(
                "Circular dependency detected when resolving {:#?}.\nResole history is:\n{:#?}",
                type_id, tracked
            );
        }

        tracked.insert(type_id);

        CycleGuard {
            guarded_type: type_id,
            stopper: &self,
        }
    }

    fn untrack(&self, type_id: &TypeId) {
        let mut tracked = self.tracked.borrow_mut();

        tracked.remove(type_id);
    }
}

pub struct CycleGuard<'a> {
    guarded_type: TypeId,
    stopper: &'a CycleStopper,
}

impl<'a> Drop for CycleGuard<'a> {
    fn drop(&mut self) {
        self.stopper.untrack(&self.guarded_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_new_types() {
        let stopper: CycleStopper = Default::default();

        stopper.track(TypeId::of::<i32>());
        stopper.track(TypeId::of::<u32>());
    }

    #[test]
    #[should_panic]
    fn panics_on_tracked_types() {
        let stopper: CycleStopper = Default::default();

        let _ = {
            let guard = stopper.track(TypeId::of::<i32>());
            let _ = stopper.track(TypeId::of::<i32>());

            guard
        };
    }

    #[test]
    fn tracked_types_can_get_untracked() {
        let stopper: CycleStopper = Default::default();

        {
            stopper.track(TypeId::of::<i32>());
        } // This goes out of scope
        stopper.track(TypeId::of::<i32>());
    }
}
