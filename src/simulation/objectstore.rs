#![allow(dead_code)]

use std::cell::RefCell;

struct Object<T> {
    pub id: usize,
    pub inner: T,
}

pub trait ObjectDefault {
    fn default() -> Self;
}

pub struct ObjectStore<T> {
    id: RefCell<usize>,
    objects: RefCell<Vec<Object<T>>>,
}

impl<T> ObjectStore<T>
where
    T: ObjectDefault + Clone,
{
    pub fn new() -> Self {
        ObjectStore {
            id: RefCell::new(0),
            objects: RefCell::new(Vec::new()),
        }
    }

    fn inc_id(&self) {
        *self.id.borrow_mut() += 1;
    }

    fn id(&self) -> usize {
        *self.id.borrow()
    }

    pub fn insert_object(&self, object: T) -> usize {
        let id = self.create_object();
        self.update_object(id, object);
        return id;
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

    pub fn create_object(&self) -> usize {
        let mut objects = self.objects.borrow_mut();
        let entity_id = self.id();
        objects.push(Object {
            id: entity_id,
            inner: T::default(),
        });
        self.inc_id();
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
        let ref_obj = &mut objects[entity_index].inner;
        return ref_obj.clone();
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

    pub fn garbage_collect_filter(&mut self, filter: impl Fn(&T) -> bool) {
        let mut objects = self.objects.borrow_mut();
        objects.retain(|x| !filter(&x.inner));
    }

    pub fn filter_ids(&self, filter: impl Fn(&T) -> bool) -> Vec<usize> {
        let objects = self.objects.borrow_mut();
        return objects
            .iter()
            .filter(|x| filter(&x.inner))
            .map(|x| x.id)
            .collect();
    }

    pub fn filter_map<U>(&self, filter: impl Fn(&T) -> Option<U>) -> Vec<U> {
        let objects = self.objects.borrow_mut();
        return objects.iter().filter_map(|x| filter(&x.inner)).collect();
    }

    pub fn with(&self, id: usize, mut f: impl FnMut(&mut T)) {
        let entity_index = self.id_to_index(id);
        let mut objects = self.objects.borrow_mut();
        f(&mut objects[entity_index].inner);
    }

    pub fn with_new(&self, mut f: impl FnMut(&mut T, usize)) {
        let entity_id = self.create_object();
        let entity_index = self.id_to_index(entity_id);
        let mut objects = self.objects.borrow_mut();
        f(&mut objects[entity_index].inner, entity_id);
    }

    pub(crate) fn len(&self) -> usize {
        self.objects.borrow().len()
    }
}
