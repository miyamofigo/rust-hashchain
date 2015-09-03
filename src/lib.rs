#![feature(ptr_as_ref)]

#[test]
fn it_works() {

  let mut node1 = Node::new(1,1);
  let node2 = Node::new(2,3);
  node1.set_next(Box::new(node2));

  assert_eq!(node1.key, 1);
  assert_eq!(node1.value, 1);

  let next_node1 = node1.next.as_ref().unwrap();
  assert_eq!(next_node1.key, 2);
  assert_eq!(next_node1.value, 3);

  let mut list1 = HashList::<u32,u32>::new();
  assert!(list1.is_empty());

  let node3 = Node::new(4,5);
  list1.replace(Box::new(node3));
  assert!(!list1.is_empty());

  let node4 = Node::new(24,35);
  let mut head = list1.head.as_mut().unwrap();
  head.set_next(Box::new(node4));

  let next_node2 
    = head.next.as_mut().unwrap();
  assert_eq!(next_node2.key, 24);
  assert_eq!(next_node2.value, 35);

  let mut list2 = HashList::<u32,u32>::new();
  let node5 = Node::new(5,666);
  list2.replace(Box::new(node5));
  let removed = list2.pop_front().unwrap();
  assert_eq!(removed.key, 5);
  assert_eq!(removed.value, 666);
  assert!(list2.is_empty());

  let node6 = Node::new(6,777);
  let mut list3 = HashList::<u32,u32>::new();
  list3.replace(Box::new(node6));
  let node7 = Node::new(7,893);
  list3.head.as_mut()
            .map(|head| head.set_next(Box::new(node7)));

  let mut iter1 = list3.iter();
  let pair1 = iter1.next();
  assert_eq!(pair1.unwrap(), (&6,&777));
  let pair2 = iter1.next();
  assert_eq!(pair2.unwrap(), (&7,&893));

  let node8 = Node::new(8,555);
  let mut list4 = HashList::<u32,u32>::new();
  list4.replace(Box::new(node8));
  let node9 = Node::new(9,1000);
  // Only tests need push method, so I haven't implemented one.
  list4.head.as_mut()
            .map(|head| head.set_next(Box::new(node9)));
  
  for (k, v) in list4.iter_mut() {
    *k = 10;
    *v = 111111;
  } 

  for (k, v) in list4.iter_mut() {
    assert_eq!(*k, 10);
    assert_eq!(*v, 111111);
  }

  let mut map = HashMap::new(256);
  map.put(1,2);
  map.put(2,3);
  assert_eq!(map.get(1).unwrap(), &2);
  assert_eq!(map.get(2).unwrap(), &3);

  map.remove(2);
  assert!(map.get(2).is_err());
      
}

type Link<K,V> = Option<Box<Node<K,V>>>;

struct Rawlink<T> { p: *mut T, }

impl<T> Copy for Rawlink<T> {}
unsafe impl<T:Send> Send for Rawlink<T> {}
unsafe impl<T:Sync> Sync for Rawlink<T> {}
 
struct HashList<K,V> {
  head: Link<K,V>,
}

struct Node<K,V> {
  key: K,
  value: V,
  next: Link<K,V>,
}

struct Iter<'a,K:'a,V:'a> {
  head: &'a Link<K,V>,
}

impl<'a,K,V> Clone for Iter<'a,K,V> {

  fn clone(&self) -> Iter<'a,K,V> {
    Iter { head: self.head.clone() }
  }

}

struct  IterMut <'a,K:'a,V:'a> {
  list: &'a mut HashList<K,V>,
  head: Rawlink<Node<K,V>>,
}

impl<T> Rawlink<T> {

  fn none() -> Rawlink<T> {
    Rawlink { p: std::ptr::null_mut() }
  }

  fn some(n: &mut T) -> Rawlink<T> {
    Rawlink { p: n }
  }

  unsafe fn resolve<'a>(&self) -> Option<&'a T> {
    self.p.as_ref()
  }

  unsafe fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
    self.p.as_mut()
  } 

  fn take(&mut self) -> Rawlink<T> {
    std::mem::replace(self, Rawlink::none())
  }

  fn is_null(&self) -> bool {
    self.p.is_null()
  }

}

impl<T> Clone for Rawlink<T> {
  
  fn clone(&self) -> Rawlink<T> {
    Rawlink { p: self.p }
  }

}

impl<'a,K,V> From<&'a mut Link<K,V>> for Rawlink<Node<K,V>> {

  fn from(node: &'a mut Link<K,V>) -> Self {

    match node.as_mut() {
      None => Rawlink::none(),
      Some(ptr) => Rawlink::some(ptr),
    }

  }

}

impl<K,V> Node<K,V> {

  fn new(k: K, v: V) -> Node<K,V> {

    Node {
      key: k, 
      value: v,
      next: None,
    }

  }

  fn set_next(&mut self, next: Box<Node<K,V>>) {
    self.next = Some(next);
  }

} 

impl<K,V> HashList<K,V> {
  
  fn new() -> HashList<K,V> {
    HashList { head: None }
  }

  fn iter(&self) -> Iter<K,V> {
    Iter { head: &self.head }
  }

  fn iter_mut(&mut self) -> IterMut<K,V> {

    IterMut { 
      head: Rawlink::from(&mut self.head),
      list: self,
    }

  }

  fn is_empty(&self) -> bool {
    self.head.is_none()
  } 

  fn replace(&mut self, head: Box<Node<K,V>>) {
    self.head = Some(head);
  }

  fn pop_front(&mut self) -> Link<K,V> {

    self.head.take()
             .map(|mut front_node| {
                    self.head = front_node.next.take();
                    front_node
                  })

  }

} 

impl<K,V> Drop for HashList<K,V> {
  
  fn drop(&mut self) {

    while let Some(mut head_) = self.head.take() {
      self.head = head_.next.take();
    } 

  }

} 

impl<'a,K,V> Iterator for Iter<'a,K,V> {

  type Item = (&'a K, &'a V);

  fn next(&mut self) -> Option<(&'a K, &'a V)> {

    match *self.head {

      None => None,
      Some(_) => self.head.as_ref()
                          .map(|head| {
                                  self.head = &head.next; 
                                  (&head.key, &head.value) 
                               }),
    }
  }

}

impl<'a,K,V> Iterator for IterMut<'a,K,V> {

  type Item = (&'a mut K, &'a mut V);
  
  fn next(&mut self) -> Option<(&'a mut K, &'a mut V)> {

    if self.head.is_null() { return None; }

    unsafe {

      self.head.resolve_mut()
               .map(|head| {
                      self.head = Rawlink::from(&mut head.next);
                      (&mut head.key, &mut head.value)
                    })
    }
  }

}

impl<'a,K,V> IterMut<'a,K,V> {

  fn peek(&mut self) -> Option<(&'a mut K, &'a mut V)> {
    
    if self.head.is_null() { return None; }

    unsafe {
      
      self.head.resolve_mut()
               .map(|head| (&mut head.key, &mut head.value))

    }
  }

  fn peek_next(&mut self) -> Option<(&'a mut K, &'a mut V)> {

    if self.head.is_null() { return None; }

    unsafe { 

      self.head.resolve_mut()
               .map(|head| {
                      let mut next = Rawlink::from(&mut head.next);
                      next.resolve_mut()
                          .map(|next| (&mut next.key, &mut next.value) )  
                          .unwrap()
                    })

    }
  }

}

pub struct HashMap {
  table_size: usize, 
  table: Vec<Box<HashList<u32,u32>>>,
} 

impl HashMap {

  pub fn new(table_size: usize) -> HashMap {

    let mut table = Vec::with_capacity(table_size);
    for _ in 0..table_size {
      table.push(Box::new(HashList::new()));
    }

    HashMap {
      table_size: table_size,
      table: table,
    }

  }

  pub fn put(&mut self, key: u32, value: u32) 
    -> Result<(), &'static str> {

    let hash = (key as usize) % self.table_size;

    let mut list = match self.table.get_mut(hash) {

                     Some(mut list) 
                       => match list.head { 
                            None => {
                              list.replace(Box::new(Node::new(key, value)));
                              return Ok(());
                            }, 
                            _ => list,
                          },
                     None => panic!("You specified a wrong key number!"),

                   }; 

    let mut iter = list.iter_mut();

    loop {

      let (k, v) = iter.next().unwrap();
      if *k == key {
        *v = value;
        return Ok(());
      } 

      unsafe { 
        match iter.head.resolve_mut() {

           Some(ref mut head) 
             => match head.next {
                  None => {
                    head.set_next(Box::new(Node::new(key, value)));
                    return Ok(());
                  },
                  _ => (),

                },
           None => return Err("Something wrong has happened.."),

        }
      }
    }
  }

  pub fn get(&self, key: u32) -> Result<&u32, &'static str> {

    let hash = (key as usize) % self.table_size;

    let list = match self.table.get(hash) {
                 Some(list) => list,
                 None => return Err("You specified a wrong key number!"),
               };

    for (k,v) in list.iter() {
      if *k == key { return Ok(v); }
    }

    Err("The key you specified does not exist.")  

  }

  pub fn remove(&mut self, key: u32) {

    let hash = (key as usize) % self.table_size;
    let mut list = match self.table.get_mut(hash) {
                     Some(mut list) => list, 
                     None => panic!("You specified a wrong key number!"),
                   }; 

    let mut iter = list.iter_mut();

    let (k, _)  = iter.peek().unwrap();
    if *k == key { 
      iter.list.pop_front(); 
      return;
    } 

    loop {

      if iter.head.is_null() { return; }
      let (k, _)  = iter.peek_next().unwrap();

      if *k == key {
      
        unsafe {

          iter.head.resolve_mut()
                   .map(|head| {
                          iter.next();
                          match unsafe { iter.head.resolve_mut() } {
                            None => panic!("Something wrong has happened"),
                            Some(current) 
                              => match current.next {
                                   None => { head.next = None; },
                                   Some(_) => { 
                                     head.set_next(current.next.take().unwrap()); 
                                   },
                                 }, 
                          }});
          return;
        
        }
      }

      iter.next();

    }
  } 

}  
