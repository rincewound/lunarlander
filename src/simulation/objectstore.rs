use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
};

struct Object<T> {
    pub id: usize,
    pub inner: T,
}

pub trait ObjectDefault {
    fn default() -> Self;
}

pub struct ObjectStore<T> {
    id: usize,
    objects: RefCell<Vec<Object<T>>>,
}

impl<T> ObjectStore<T>
where
    T: ObjectDefault + Clone,
{
    pub fn new() -> Self {
        ObjectStore {
            id: 0,
            objects: RefCell::new(Vec::new()),
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(&mut T, usize)) {
        let mut objects = self.objects.borrow_mut();
        for e in objects.iter_mut() {
            f(&mut e.inner, e.id);
        }
    }

    fn id_to_index(&self, id: usize) -> usize {
        let mut idx = 0;
        let objects = self.objects.borrow_mut();
        for e in objects.iter() {
            if e.id == id {
                return idx;
            }
            idx += 1;
        }
        panic!("Entity with id {} does not exist", id)
    }

    pub fn create_object(&mut self) -> usize {
        let mut objects = self.objects.borrow_mut();
        let entity_id = self.id;
        self.id += 1;
        objects.push(Object {
            id: entity_id,
            inner: T::default(),
        });
        return entity_id;
    }

    pub fn get_object(&self, id: usize) -> T {
        let entity_index = self.id_to_index(id);
        let objects = self.objects.borrow_mut();
        return objects[entity_index].inner.clone();
    }

    pub fn get_object_clone(&self, id: usize) -> T {
        let entity_index = self.id_to_index(id);
        let mut objects = self.objects.borrow_mut();
        let refObj = &mut objects[entity_index].inner;
        return refObj.clone();
    }

    pub fn update_object(&self, id: usize, object: T) {
        let entity_index = self.id_to_index(id);
        let mut objects = self.objects.borrow_mut();
        objects[entity_index].inner = object;
    }

    pub fn garbage_collect(&mut self, ids_to_remove: &Vec<usize>) {
        let mut objects = self.objects.borrow_mut();
        objects.retain(|x| !ids_to_remove.contains(&x.id));
    }
}
