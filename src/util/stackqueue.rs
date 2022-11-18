pub struct StackDequeue<T, const CAP: usize>
where
    T: Default + Copy,
{
    data: [T; CAP],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T, const CAP: usize> StackDequeue<T, CAP>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            data: [T::default(); CAP],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        return CAP;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let idx = self.head;
            self.head += 1;
            if self.head == self.data.len() {
                self.head = 0;
            }
            self.count -= 1;

            let res = unsafe { self.data.get_unchecked(idx) };
            Some(*res)
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            if self.tail == 0 {
                self.tail = self.data.len() - 1;
            } else {
                self.tail -= 1
            }
            self.count -= 1;

            let res = unsafe { self.data.get_unchecked(self.tail) };
            Some(*res)
        }
    }

    pub fn push_back(&mut self, v: T) {
        if self.count == self.data.len() {
            panic!("overfilled StackDequeue")
        } else {
            let slot = unsafe { self.data.get_unchecked_mut(self.tail) };
            *slot = v;
            self.tail += 1;
            if self.tail == self.data.len() {
                self.tail = 0;
            }
            self.count += 1;
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T, CAP> {
        Iter { pos: 0, q: self }
    }
}

pub struct Iter<'a, T, const CAP: usize>
where
    T: 'a + Default + Copy,
{
    pos: usize,
    q: &'a StackDequeue<T, CAP>,
}

impl<'a, T, const CAP: usize> std::iter::Iterator for Iter<'a, T, CAP>
where
    T: Default + Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.q.count {
            return None;
        }
        let old_pos = self.pos;
        self.pos += 1;
        Some(&self.q[old_pos])
    }
}

impl<T, const CAP: usize> std::ops::Index<usize> for StackDequeue<T, CAP>
where
    T: Default + Copy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.count {
            panic!(
                "index out of bounds: {} but stackdequeue length is only {}",
                index, self.count
            );
        }
        let mut idx = self.head + index;
        if idx >= self.data.len() {
            idx -= self.data.len();
        }

        &self.data[idx]
    }
}

impl<T, const CAP: usize> std::ops::IndexMut<usize> for StackDequeue<T, CAP>
where
    T: Default + Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.count {
            panic!(
                "index out of bounds: {} but stackdequeue length is only {}",
                index, self.count
            );
        }
        let mut idx = self.head + index;
        if idx >= self.data.len() {
            idx -= self.data.len();
        }

        &mut self.data[idx]
    }
}

#[test]
fn test_push_back_pop_front() {
    let mut q: StackDequeue<usize, 1024> = StackDequeue::new();

    assert_eq!(q.pop_front(), None);

    for i in 0..q.capacity() {
        q.push_back(i);
        assert_eq!(q.len(), i + 1);
    }

    for i in 0..q.capacity() {
        assert_eq!(q.pop_front(), Some(i));
        assert_eq!(q.len(), q.capacity() - i - 1);
    }

    assert_eq!(q.pop_front(), None);
}

#[test]
fn test_push_back_pop_back() {
    let mut q: StackDequeue<usize, 1024> = StackDequeue::new();

    assert_eq!(q.pop_front(), None);

    for i in 0..q.capacity() {
        q.push_back(i);
        assert_eq!(q.len(), i + 1);
    }

    for i in 0..q.capacity() {
        assert_eq!(q.pop_back(), Some(q.capacity() - i - 1));
        assert_eq!(q.len(), q.capacity() - i - 1);
    }

    assert_eq!(q.pop_front(), None);
}
